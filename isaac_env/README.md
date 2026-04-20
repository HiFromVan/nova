# Isaac Lab 仿真服务器

这个目录包含运行在 Windows (RTX 3090) 上的 Isaac Lab 仿真服务器。

## 快速开始（Windows）

### 1. 安装依赖

```powershell
# 安装 Python 依赖
pip install -r requirements.txt

# 生成 gRPC 代码
bash generate_proto.sh
# 或者在 Windows 上：
python -m grpc_tools.protoc -I../proto --python_out=./proto --grpc_python_out=./proto ../proto/simulator.proto
```

### 2. 启动服务器

```powershell
python server.py
```

应该看到：
```
============================================================
Isaac Lab Simulator Server
============================================================
服务器已启动
监听地址: 0.0.0.0:50051
等待 Nova 客户端连接...
============================================================
```

### 3. 配置防火墙

```powershell
# 以管理员身份运行
New-NetFirewallRule -DisplayName "Isaac Lab gRPC" -Direction Inbound -Protocol TCP -LocalPort 50051 -Action Allow
```

### 4. 获取 IP 地址

```powershell
ipconfig
```

记下 IPv4 地址（例如：192.168.1.100），Mac 端需要用这个地址连接。

## 测试连接

### 本地测试

```powershell
# 安装 grpcurl
choco install grpcurl

# 测试连接
grpcurl -plaintext localhost:50051 list
```

### 从 Mac 测试

在 Mac 上运行：
```bash
grpcurl -plaintext <Windows-IP>:50051 list
```

## 项目结构

```
isaac_env/
├── README.md              # 本文件
├── requirements.txt       # Python 依赖
├── server.py             # gRPC 服务器
├── generate_proto.sh     # 生成 gRPC 代码脚本
├── proto/                # 生成的 gRPC 代码
│   ├── simulator_pb2.py
│   └── simulator_pb2_grpc.py
└── humanoid_env.py       # Isaac Lab 环境（待实现）
```

## 下一步

1. 安装 Isaac Lab（见 `docs/isaac_lab_setup.md`）
2. 实现 `humanoid_env.py`（人形机器人环境）
3. 在 `server.py` 中集成 Isaac Lab
4. 测试完整流程

## 故障排查

### 端口被占用

```powershell
netstat -ano | findstr :50051
taskkill /PID <PID> /F
```

### 防火墙阻止连接

1. 检查 Windows Defender 防火墙设置
2. 确保端口 50051 已开放
3. 尝试临时关闭防火墙测试

### 无法从 Mac 连接

1. 确保两台电脑在同一局域网
2. ping Windows IP 地址
3. 检查路由器设置
