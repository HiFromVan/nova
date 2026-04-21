# Nova

Nova 是人形机器人的运动控制层，与 [Becoming](https://github.com/HiFromVan/becoming) 认知系统配合，构成完整的自主人形机器人栈。

```
任何硬件  +  Nova  +  Becoming  =  通用人形机器人
```

## 架构

```
Becoming（Python，认知大脑）
  视觉 + 语言 + 任务规划，1-10Hz
        │ gRPC（本机）
        ▼
Nova（Rust，运动身体）
  指令解析 → 步态规划 → 安全守卫，50-500Hz
        │
   ┌────┴────┐
   │仿真      │真机
   │gRPC     │DDS
   ▼         ▼
Isaac Lab   宇树 H1
（开发用）   （生产）
```

- **Becoming** 决定做什么，Nova 决定怎么做
- **Nova** 是硬件无关的：换机器人品牌只换底层驱动，Becoming 不变
- **Isaac Lab** 仅用于开发验证，不是生产组件

## 快速开始

### 仿真模式（Mac + Windows）

```bash
# Windows — 启动 Isaac Lab 仿真服务器
cd nova/isaac_env
python server.py

# Mac — 启动 Nova 连接仿真
cargo run -- --sim-addr http://<Windows-IP>:50051
```

### 真机模式（宇树 H1）

```bash
# 确保与 H1 在同一网段（H1 默认 192.168.123.161）
cargo run -- --real --domain 0
```

## 项目结构

```
nova/
├── src/
│   ├── main.rs              入口，控制环路
│   ├── becoming/            接收 Becoming 指令的 gRPC server
│   ├── robot/
│   │   ├── mod.rs           RobotIO trait（硬件抽象）
│   │   ├── sim.rs           Isaac Lab gRPC（仿真）
│   │   └── unitree.rs       CycloneDDS（宇树真机）
│   ├── control/             控制环路 + 安全守卫
│   └── brain_interface/     BrainInterface trait + BaselineGait
├── isaac_env/
│   └── server.py            Isaac Lab gRPC 服务器（Windows 运行）
├── proto/
│   ├── simulator.proto      Nova ↔ Isaac Lab 接口
│   └── nova_control.proto   Becoming ↔ Nova 接口
└── demo/
    └── client.py            Demo 客户端（模拟 Becoming 发指令）
```

## 文档

| 文档 | 内容 |
|------|------|
| [架构设计](docs/architecture.md) | 完整架构图、频率层级、开发态 vs 生产态 |
| [Becoming 设计](docs/becoming.md) | VLA 模型架构、接口定义、开发阶段规划 |
| [部署指南](docs/deployment.md) | 硬件配置、网络配置、数据流 |
| [路线图](docs/roadmap.md) | 当前进度、各阶段目标 |
| [Isaac Lab 安装](docs/isaac_lab_setup.md) | Windows 端安装步骤 |
| [学习资料](docs/resources.md) | 参考项目、论文、工具 |

## 当前状态

见 [路线图](docs/roadmap.md)。当前目标：实现 Becoming ↔ Nova gRPC 接口 + Demo 客户端，跑通完整仿真闭环。

## License

MIT
