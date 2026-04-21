# Nova 部署架构

## 开发态（现在）

```
Mac M4 Pro
┌─────────────────────────────────────────────┐
│  Becoming（Python）          Nova（Rust）    │
│  认知决策  ──gRPC(本机)──▶  运动控制         │
└─────────────────────────────┬───────────────┘
                              │ gRPC（局域网）
Windows RTX 3090
┌─────────────────────────────▼───────────────┐
│  Isaac Lab（仿真）                           │
│  物理仿真验证，不是生产组件                   │
└─────────────────────────────────────────────┘
```

**Isaac Lab 只用于仿真验证，生产环境不存在这一层。**

---

## 生产态（真机）

```
H1 板载电脑
┌─────────────────────────────────────────────┐
│  Becoming（板载GPU）         Nova（板载CPU） │
│  VLA推理 1-10Hz ──gRPC──▶  运动控制 500Hz   │
└─────────────────────────────┬───────────────┘
                              │ DDS（CycloneDDS）
H1 硬件层
┌─────────────────────────────▼───────────────┐
│  电机控制器（1kHz）                          │
│  IMU / 关节编码器 / 足力传感器               │
└─────────────────────────────────────────────┘

Mac（可选，不在控制环路）
└── 监控面板 / 遥操作 / 日志采集
```

---

## 硬件配置

| 设备 | 用途 | 运行内容 |
|------|------|---------|
| Mac M4 Pro | 开发机 | Becoming + Nova（开发阶段） |
| Windows RTX 3090 | 仿真服务器 | Isaac Lab（仿真阶段） |
| H1 板载电脑 | 生产运行 | Becoming + Nova（生产阶段） |

---

## 通信协议

| 链路 | 协议 | 延迟 | 用途 |
|------|------|------|------|
| Becoming ↔ Nova | gRPC（本机 IPC） | <1ms | 指令下发 + 状态反馈 |
| Nova ↔ Isaac Lab | gRPC（局域网） | ~5ms | 仿真（开发用） |
| Nova ↔ H1 硬件 | DDS（CycloneDDS） | <0.5ms | 真机控制 |

---

## 数据流

### 仿真阶段
```
键盘/Becoming → Nova → gRPC → Isaac Lab
                              物理仿真一步
               Nova ← gRPC ← SensorData
```

### 真机阶段
```
Becoming → Nova（50Hz 运动规划）
               ↓
           Nova（500Hz 控制环路）
               ↓ DDS rt/lowcmd
           H1 电机控制器
               ↓ DDS rt/lowstate
           Nova 读取传感器 → 反馈给 Becoming
```

---

## 网络配置（开发阶段）

### Windows 端
```powershell
# 开放防火墙
netsh advfirewall firewall add rule name="Isaac Lab gRPC" dir=in action=allow protocol=TCP localport=50051

# 启动仿真服务器
cd nova/isaac_env
python server.py
```

### Mac 端
```bash
# 测试连通性
grpcurl -plaintext <Windows-IP>:50051 list

# 启动 Nova（连仿真）
cargo run -- --sim-addr http://<Windows-IP>:50051
```

### 真机
```bash
# 确保 Mac/板载电脑与 H1 在同一网段
# H1 默认 IP：192.168.123.161
cargo run -- --real --domain 0
```
