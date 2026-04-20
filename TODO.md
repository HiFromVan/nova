# Nova TODO

## 已完成

- [x] 项目基础架构（Rust + Bevy + Rapier3D）
- [x] `BrainInterface` trait 定义
- [x] `BaselineGait` 规则步态实现
- [x] Isaac Lab 环境搭建（Windows, RTX 3090, Isaac Sim 4.5）
- [x] H1 人形机器人 demo 跑通（`h1_locomotion.py`）
- [x] gRPC proto 定义（`proto/simulator.proto`）
- [x] Python gRPC server 骨架（`isaac_env/server.py`）
- [x] Rust gRPC client 骨架（`examples/grpc_client.rs`）
- [x] Rust ↔ Python gRPC 通信链路验证（mock 数据）
- [x] Isaac Lab server 接入真实 H1 仿真环境

## 进行中

- [ ] 端到端测试：Rust client → Isaac Lab server → H1 仿真
- [ ] 验证 `desired_velocity` 指令能驱动机器人运动

## 近期

### gRPC 通信层
- [ ] Rust 端封装 `SimulatorClient` 为可复用模块（`src/simulation/grpc_client.rs`）
- [ ] 实现 `StreamStep` 流式通信（比单步 Step 延迟更低）
- [ ] 错误处理：连接断开自动重连

### BrainInterface 接入仿真
- [ ] 把 `BrainInterface::decide()` 的输出接到 gRPC `Step` 请求
- [ ] `BaselineGait` 驱动 H1 在仿真里行走
- [ ] 仿真状态（`SensorData`）回传给 `BrainInterface`

### 仿真环境
- [ ] 支持多环境并发（当前 server 单环境）
- [ ] 暴露 reset 接口给 Rust 端
- [ ] 仿真速度控制（实时 / 加速）

## 中期

- [ ] 设计 Nova 自己的机器人 URDF（关节数、尺寸）
- [ ] 把 H1 模型替换为 Nova 专属模型
- [ ] 针对 Nova 模型重新训练底层步态策略
- [ ] Mac 端 Rust 通过网络连接 Windows Isaac Lab（跨机器 gRPC）

## 长期

- [ ] Becoming 决策系统接入 `BrainInterface`
- [ ] 视觉输入（摄像头观测）
- [ ] 真实硬件适配层（电机驱动、IMU 读取）
- [ ] Sim-to-real 迁移
