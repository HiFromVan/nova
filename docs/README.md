# Nova 项目文档

## 项目概述

Nova 是一个人形机器人仿真和控制系统，设计用于连接 Becoming 决策系统。

## 架构

- **Mac (M4 Pro)**: 开发机，运行 Nova (Rust) + Becoming (决策系统)
- **Windows (RTX 3090)**: 仿真服务器，运行 Isaac Lab
- **通信**: gRPC (端口 50051)

## 文档索引

### 1. [架构设计](architecture.md)
- 系统架构图
- 为什么选择 Isaac Lab
- 下一步计划

### 2. [部署架构](deployment.md)
- 硬件配置
- 网络配置
- 数据流说明
- 故障排查

### 3. [Isaac Lab 安装](isaac_lab_setup.md)
- Windows 端安装步骤
- CUDA 和驱动配置
- 防火墙设置
- 性能优化

## 快速开始

### Windows 端（仿真服务器）

```powershell
# 1. 安装 Isaac Lab（见 isaac_lab_setup.md）

# 2. 安装依赖
cd nova/isaac_env
pip install -r requirements.txt

# 3. 生成 gRPC 代码
python -m grpc_tools.protoc -I../proto --python_out=./proto --grpc_python_out=./proto ../proto/simulator.proto

# 4. 启动服务器
python server.py

# 5. 获取 IP 地址
ipconfig
```

### Mac 端（开发机）

```bash
# 1. 测试连接（替换为 Windows IP）
ping 192.168.x.x
grpcurl -plaintext 192.168.x.x:50051 list

# 2. 运行 Nova（待实现 gRPC 客户端）
cd nova
cargo run
```

## 项目结构

```
nova/
├── src/                    # Rust 代码
│   ├── brain_interface/    # BrainInterface trait
│   ├── control/           # 控制系统
│   ├── models/            # 机器人模型
│   └── main.rs
├── isaac_env/              # Isaac Lab 环境（Windows）
│   ├── server.py          # gRPC 服务器
│   ├── requirements.txt   # Python 依赖
│   └── README.md
├── proto/                  # gRPC 协议定义
│   └── simulator.proto
├── docs/                   # 文档
│   ├── README.md          # 本文件
│   ├── architecture.md    # 架构设计
│   ├── deployment.md      # 部署架构
│   └── isaac_lab_setup.md # 安装指南
└── Cargo.toml
```

## 当前状态

### ✅ 已完成
- [x] BrainInterface trait 设计
- [x] BaselineGait 实现
- [x] 键盘控制系统
- [x] gRPC 协议定义
- [x] Isaac Lab 服务器框架
- [x] 完整文档

### 🚧 进行中
- [ ] Isaac Lab 环境实现
- [ ] Nova gRPC 客户端（Rust）
- [ ] 端到端测试

### 📋 待办
- [ ] 人形机器人模型（URDF/USD）
- [ ] Becoming 集成
- [ ] 性能优化
- [ ] 可视化界面

## 下一步

1. **在 Windows 上安装 Isaac Lab**
   - 按照 `docs/isaac_lab_setup.md` 操作
   - 测试 GPU 和 CUDA

2. **启动仿真服务器**
   - 运行 `isaac_env/server.py`
   - 配置防火墙
   - 获取 IP 地址

3. **实现 Nova gRPC 客户端**
   - 添加 tonic (Rust gRPC) 依赖
   - 生成 Rust gRPC 代码
   - 实现客户端连接

4. **测试连接**
   - Mac 连接到 Windows
   - 发送测试指令
   - 验证数据流

## 联系和支持

如有问题，请查看：
- `docs/deployment.md` - 部署问题
- `docs/isaac_lab_setup.md` - 安装问题
- `isaac_env/README.md` - 服务器问题
