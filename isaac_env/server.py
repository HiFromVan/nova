"""
Isaac Lab gRPC 服务器

这个服务器运行在 Windows (RTX 3090) 上，提供人形机器人仿真服务。
Mac 上的 Nova 通过 gRPC 连接到这个服务器。
"""

import grpc
from concurrent import futures
import time
import numpy as np
import sys
import os

# 添加 proto 目录到路径
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'proto'))

# 导入生成的 gRPC 代码
import simulator_pb2
import simulator_pb2_grpc

# TODO: 导入 Isaac Lab
# from omni.isaac.lab.app import AppLauncher
# from humanoid_env import HumanoidEnv


class SimulatorServicer(simulator_pb2_grpc.SimulatorServicer):
    """仿真器服务实现"""
    
    def __init__(self):
        print("初始化 Isaac Lab 环境...")
        # TODO: 初始化 Isaac Lab 环境
        # self.env = HumanoidEnv()
        self.step_count = 0
        print("环境初始化完成")
    
    def Step(self, request, context):
        """单步仿真"""
        self.step_count += 1
        
        # 解析控制指令
        joint_targets = list(request.joint_targets)
        desired_velocity = request.desired_velocity
        
        print(f"Step {self.step_count}: 接收到 {len(joint_targets)} 个关节目标")
        
        # TODO: 执行仿真步骤
        # obs = self.env.step(joint_targets)
        
        # 临时返回模拟数据
        response = simulator_pb2.SensorData(
            timestamp=time.time(),
            position=simulator_pb2.Vec3(x=0.0, y=1.0, z=0.0),
            orientation=simulator_pb2.Quat(x=0.0, y=0.0, z=0.0, w=1.0),
            velocity=simulator_pb2.Vec3(x=0.0, y=0.0, z=0.0),
            angular_velocity=simulator_pb2.Vec3(x=0.0, y=0.0, z=0.0),
            joint_angles=joint_targets if joint_targets else [0.0] * 8,
            joint_velocities=[0.0] * 8,
            foot_contacts=[True, True],
            is_stable=True
        )
        
        return response
    
    def Reset(self, request, context):
        """重置环境"""
        print("重置环境...")
        self.step_count = 0
        
        # TODO: 重置 Isaac Lab 环境
        # obs = self.env.reset()
        
        response = simulator_pb2.SensorData(
            timestamp=time.time(),
            position=simulator_pb2.Vec3(x=0.0, y=1.0, z=0.0),
            orientation=simulator_pb2.Quat(x=0.0, y=0.0, z=0.0, w=1.0),
            velocity=simulator_pb2.Vec3(x=0.0, y=0.0, z=0.0),
            angular_velocity=simulator_pb2.Vec3(x=0.0, y=0.0, z=0.0),
            joint_angles=[0.0] * 8,
            joint_velocities=[0.0] * 8,
            foot_contacts=[True, True],
            is_stable=True
        )
        
        return response
    
    def StreamStep(self, request_iterator, context):
        """流式仿真（高性能模式）"""
        print("开始流式仿真...")
        
        for request in request_iterator:
            # 处理每个请求
            response = self.Step(request, context)
            yield response


def serve():
    """启动 gRPC 服务器"""
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    simulator_pb2_grpc.add_SimulatorServicer_to_server(
        SimulatorServicer(), server
    )
    
    # 监听所有网络接口的 50051 端口
    server.add_insecure_port('[::]:50051')
    server.start()
    
    print("=" * 60)
    print("Isaac Lab Simulator Server")
    print("=" * 60)
    print("服务器已启动")
    print("监听地址: 0.0.0.0:50051")
    print("等待 Nova 客户端连接...")
    print("=" * 60)
    
    try:
        server.wait_for_termination()
    except KeyboardInterrupt:
        print("\n正在关闭服务器...")
        server.stop(0)


if __name__ == '__main__':
    serve()
