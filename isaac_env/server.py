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
import queue
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
NUM_JOINTS = 19

# PhysX 不是线程安全的，所有仿真操作必须在主线程执行
# gRPC 线程通过此队列把工作提交给主线程
_work_queue: queue.Queue = queue.Queue()


class _WorkItem:
    """单次仿真操作，gRPC 线程提交后阻塞等待结果"""
    def __init__(self, fn):
        self._fn = fn
        self._done = threading.Event()
        self.result = None
        self.exc = None

    def run(self):
        try:
            self.result = self._fn()
        except Exception as e:
            self.exc = e
        finally:
            self._done.set()

    def wait(self):
        self._done.wait()
        if self.exc:
            raise self.exc
        return self.result


def build_env():
    from isaaclab_tasks.utils.parse_cfg import load_cfg_from_registry

    agent_cfg = load_cfg_from_registry(TASK, "rsl_rl_cfg_entry_point")
    agent_cfg = handle_deprecated_rsl_rl_cfg(agent_cfg, metadata.version("rsl-rl-lib"))

    env_cfg = H1RoughEnvCfg_PLAY()
    env_cfg.scene.num_envs = 1
    env_cfg.episode_length_s = 1_000_000
    env_cfg.curriculum = None
    env_cfg.sim.device = "cuda:0"

    env = RslRlVecEnvWrapper(ManagerBasedRLEnv(cfg=env_cfg))
    device = env.unwrapped.device

    checkpoint = get_published_pretrained_checkpoint(RL_LIBRARY, TASK)
    runner = OnPolicyRunner(env, agent_cfg.to_dict(), log_dir=None, device=device)
    loaded = torch.load(checkpoint, map_location=device)

    src = loaded.get("model_state_dict", loaded.get("actor_state_dict", {}))
    if any(k.startswith("actor.") for k in src):
        actor_sd, critic_sd = {}, {}
        for k, v in src.items():
            if k.startswith("actor."):
                actor_sd["mlp." + k[len("actor."):]] = v
            elif k.startswith("critic."):
                critic_sd["mlp." + k[len("critic."):]] = v
            elif k == "std":
                actor_sd["distribution.std_param"] = v
        loaded["actor_state_dict"]  = actor_sd
        loaded["critic_state_dict"] = critic_sd
        loaded.pop("model_state_dict", None)

    patched = checkpoint + ".patched.pt"
    torch.save(loaded, patched)
    runner.load(patched)
    policy = runner.get_inference_policy(device=device)

    return env, policy, device


class SimulatorServicer(simulator_pb2_grpc.SimulatorServicer):

    def __init__(self, env, policy, device):
        self.env = env
        self.policy = policy
        self.device = device
        self.obs = None
        # 初始 reset 在主线程直接执行（此时主循环还未启动，不能走队列）
        self._do_reset()
        print("[Server] 环境就绪，等待 gRPC 指令...")

    def _state_to_proto(self):
        robot = self.env.unwrapped.scene["robot"]
        d = robot.data
        pos  = d.root_pos_w[0]
        quat = d.root_quat_w[0]
        vel  = d.root_lin_vel_w[0]
        ang  = d.root_ang_vel_w[0]
        jpos = d.joint_pos[0]
        jvel = d.joint_vel[0]
        stable = bool(pos[2].item() > 0.3)
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
            foot_contacts=[stable, stable],
            is_stable=stable,
        )

    def _do_step(self, request):
        # 如果 Nova 发送了关节角度，直接使用（高层模式）
        if request.joint_targets and len(request.joint_targets) > 0:
            # Nova 发送的是 8 个关节，需要映射到 Isaac Lab 的 19 个关节
            # 暂时只使用前 8 个关节，其余保持当前值
            action = torch.zeros((1, NUM_JOINTS), device=self.device)
            for i, target in enumerate(request.joint_targets[:min(8, NUM_JOINTS)]):
                action[0, i] = target
            print(f"[Server] 使用 Nova 关节角度: {request.joint_targets[:4]}")
        else:
            # 否则使用 RL policy（fallback）
            dv = request.desired_velocity
            if dv is not None:
                self.obs[:, 9]  = dv.x
                self.obs[:, 10] = dv.y
                self.obs[:, 11] = dv.z
            with torch.inference_mode():
                action = self.policy(self.obs)
            print(f"[Server] 使用 RL policy，速度: ({dv.x if dv else 0:.2f}, {dv.y if dv else 0:.2f}, {dv.z if dv else 0:.2f})")

        self.obs, _, _, _ = self.env.step(action)
        return self._state_to_proto()

    def _do_reset(self):
        obs, _ = self.env.reset()
        self.obs = obs
        return self._state_to_proto()

    def _dispatch(self, fn):
        item = _WorkItem(fn)
        _work_queue.put(item)
        return item.wait()

    def Step(self, request, context):
        return self._dispatch(lambda: self._do_step(request))

    def Reset(self, request, context):
        return self._dispatch(self._do_reset)

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

    try:
        while simulation_app.is_running():
            # 先把队列里所有待执行的仿真操作跑完（在主线程）
            while True:
                try:
                    item = _work_queue.get_nowait()
                    item.run()
                except queue.Empty:
                    break
            # 保持 UI / 渲染响应；env.step() 内部已调用 sim.step()，
            # 空闲时才调用 update() 维持窗口
            simulation_app.update()
    except KeyboardInterrupt:
        pass
    finally:
        server.stop(0)
        simulation_app.close()
