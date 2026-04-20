"""
Isaac Lab gRPC 服务器 - 接入真实 H1 仿真环境
"""

import sys
import os

# Isaac Lab 必须在最开始初始化 SimulationApp
from isaaclab.app import AppLauncher

import argparse
parser = argparse.ArgumentParser()
parser.add_argument("--num_envs", type=int, default=1)
AppLauncher.add_app_launcher_args(parser)
args_cli, _ = parser.parse_known_args()
args_cli.headless = False  # 显示窗口
app_launcher = AppLauncher(args_cli)
simulation_app = app_launcher.app

# --- 以下 import 必须在 SimulationApp 启动后 ---
import torch
import grpc
import time
import threading
import importlib.metadata as metadata
from concurrent import futures

sys.path.insert(0, os.path.dirname(__file__))
import simulator_pb2
import simulator_pb2_grpc

from rsl_rl.runners import OnPolicyRunner
from isaaclab.envs import ManagerBasedRLEnv
from isaaclab_rl.rsl_rl import RslRlVecEnvWrapper, handle_deprecated_rsl_rl_cfg
from isaaclab_rl.utils.pretrained_checkpoint import get_published_pretrained_checkpoint
from isaaclab_tasks.manager_based.locomotion.velocity.config.h1.rough_env_cfg import H1RoughEnvCfg_PLAY

TASK = "Isaac-Velocity-Rough-H1-v0"
RL_LIBRARY = "rsl_rl"
NUM_JOINTS = 19  # H1 关节数


def build_env():
    """初始化 H1 环境和策略"""
    # 加载 agent 配置
    from isaaclab_tasks.utils.parse_cfg import load_cfg_from_registry
    agent_cfg = load_cfg_from_registry(TASK, "rsl_rl_cfg_entry_point")
    agent_cfg = handle_deprecated_rsl_rl_cfg(agent_cfg, metadata.version("rsl-rl-lib"))

    # 创建环境（单环境，用于 gRPC 控制）
    env_cfg = H1RoughEnvCfg_PLAY()
    env_cfg.scene.num_envs = 1
    env_cfg.episode_length_s = 1_000_000
    env_cfg.curriculum = None

    env = RslRlVecEnvWrapper(ManagerBasedRLEnv(cfg=env_cfg))
    device = env.unwrapped.device

    # 加载预训练策略
    checkpoint = get_published_pretrained_checkpoint(RL_LIBRARY, TASK)
    runner = OnPolicyRunner(env, agent_cfg.to_dict(), log_dir=None, device=device)
    runner.load(checkpoint)
    policy = runner.get_inference_policy(device=device)

    return env, policy, device


class SimulatorServicer(simulator_pb2_grpc.SimulatorServicer):

    def __init__(self, env, policy, device):
        self.env = env
        self.policy = policy
        self.device = device
        self.obs = None
        self._lock = threading.Lock()

        # 初始 reset
        obs, _ = self.env.reset()
        self.obs = obs
        print("[Server] 环境就绪，等待 gRPC 指令...")

    def _state_to_proto(self):
        """把当前机器人状态转成 SensorData proto"""
        robot = self.env.unwrapped.scene["robot"]
        d = robot.data

        pos = d.root_pos_w[0]       # (3,)
        quat = d.root_quat_w[0]     # (4,) xyzw
        vel = d.root_lin_vel_w[0]   # (3,)
        ang = d.root_ang_vel_w[0]   # (3,)
        jpos = d.joint_pos[0]       # (19,)
        jvel = d.joint_vel[0]       # (19,)

        # 足部接触：ankle_link 索引 3,8（来自日志）
        contact = self.env.unwrapped.scene["contact_forces"]
        forces = contact.data.net_forces_w[0]  # (num_bodies, 3)
        left_contact = bool(forces[3].norm() > 1.0)
        right_contact = bool(forces[8].norm() > 1.0)

        # 稳定性：torso 没有接触地面
        torso_contact = bool(forces[9].norm() > 1.0)
        is_stable = not torso_contact

        return simulator_pb2.SensorData(
            timestamp=time.time(),
            position=simulator_pb2.Vec3(x=pos[0].item(), y=pos[1].item(), z=pos[2].item()),
            orientation=simulator_pb2.Quat(
                x=quat[0].item(), y=quat[1].item(),
                z=quat[2].item(), w=quat[3].item()
            ),
            velocity=simulator_pb2.Vec3(x=vel[0].item(), y=vel[1].item(), z=vel[2].item()),
            angular_velocity=simulator_pb2.Vec3(x=ang[0].item(), y=ang[1].item(), z=ang[2].item()),
            joint_angles=[jpos[i].item() for i in range(NUM_JOINTS)],
            joint_velocities=[jvel[i].item() for i in range(NUM_JOINTS)],
            foot_contacts=[left_contact, right_contact],
            is_stable=is_stable,
        )

    def Step(self, request, context):
        with self._lock:
            # 如果 Rust 发来了期望速度，注入到观测的 velocity_commands（索引 9:12）
            dv = request.desired_velocity
            if dv is not None:
                self.obs[:, 9] = dv.x   # lin_vel_x
                self.obs[:, 10] = dv.y  # lin_vel_y
                self.obs[:, 11] = dv.z  # ang_vel_z

            with torch.inference_mode():
                action = self.policy(self.obs)
                self.obs, _, _, _ = self.env.step(action)

            return self._state_to_proto()

    def Reset(self, request, context):
        with self._lock:
            obs, _ = self.env.reset()
            self.obs = obs
            return self._state_to_proto()

    def StreamStep(self, request_iterator, context):
        for request in request_iterator:
            yield self.Step(request, context)


def serve(env, policy, device):
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=4))
    simulator_pb2_grpc.add_SimulatorServicer_to_server(
        SimulatorServicer(env, policy, device), server
    )
    server.add_insecure_port("[::]:50051")
    server.start()
    print("=" * 50)
    print("Isaac Lab gRPC Server 已启动 :50051")
    print("=" * 50)
    return server


if __name__ == "__main__":
    env, policy, device = build_env()
    server = serve(env, policy, device)

    # 主循环：保持 simulation_app 运行
    try:
        while simulation_app.is_running():
            simulation_app.update()
    except KeyboardInterrupt:
        pass
    finally:
        server.stop(0)
        simulation_app.close()
