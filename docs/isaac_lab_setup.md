# Isaac Lab 安装指南 (Windows + RTX 3090)

## 系统要求

- Windows 10/11
- NVIDIA RTX 3090
- Python 3.10+
- CUDA 12.x
- 至少 20GB 磁盘空间

## 安装步骤

### 1. 安装 NVIDIA 驱动和 CUDA

```powershell
# 检查 NVIDIA 驱动
nvidia-smi

# 应该显示 CUDA 版本 12.x 和 GPU 信息
```

如果没有安装，从 NVIDIA 官网下载：
- https://www.nvidia.com/Download/index.aspx

### 2. 安装 Miniconda

下载并安装 Miniconda:
- https://docs.conda.io/en/latest/miniconda.html

```powershell
# 创建 Python 环境
conda create -n isaaclab python=3.10
conda activate isaaclab
```

### 3. 安装 Isaac Lab

```powershell
# 克隆 Isaac Lab 仓库
git clone https://github.com/isaac-sim/IsaacLab.git
cd IsaacLab

# 安装依赖
pip install -e .

# 验证安装
python -c "import omni.isaac.lab; print('Isaac Lab installed successfully!')"
```

### 4. 安装 gRPC 依赖

```powershell
pip install grpcio grpcio-tools
```

## 项目结构

在 Nova 项目中创建 Isaac Lab 环境：

```
nova/
├── isaac_env/              # Isaac Lab 环境（Windows）
│   ├── requirements.txt    # Python 依赖
│   ├── server.py          # gRPC 服务器
│   ├── humanoid_env.py    # 人形机器人环境
│   ├── proto/             # gRPC 协议文件
│   │   └── simulator_pb2.py
│   └── assets/            # 机器人模型
│       └── humanoid.usd
```

## 运行服务器

```powershell
# 激活环境
conda activate isaaclab

# 启动 gRPC 服务器
cd nova/isaac_env
python server.py

# 应该看到：
# Isaac Lab Simulator Server started on 0.0.0.0:50051
# Waiting for connections...
```

## 防火墙配置

### 方法 1: Windows Defender 防火墙

1. 打开 "Windows Defender 防火墙"
2. 点击 "高级设置"
3. 点击 "入站规则" → "新建规则"
4. 选择 "端口" → "TCP" → 输入 "50051"
5. 选择 "允许连接"
6. 完成

### 方法 2: PowerShell 命令

```powershell
# 以管理员身份运行
New-NetFirewallRule -DisplayName "Isaac Lab gRPC" -Direction Inbound -Protocol TCP -LocalPort 50051 -Action Allow
```

## 测试连接

### 在 Windows 上测试

```powershell
# 安装 grpcurl
choco install grpcurl

# 测试本地连接
grpcurl -plaintext localhost:50051 list
```

### 从 Mac 测试

```bash
# 获取 Windows IP 地址（在 Windows 上运行）
ipconfig

# 在 Mac 上测试连接
ping <Windows-IP>
grpcurl -plaintext <Windows-IP>:50051 list
```

## 常见问题

### 1. CUDA 版本不匹配

```powershell
# 检查 CUDA 版本
nvcc --version

# 如果版本不对，重新安装 CUDA Toolkit
```

### 2. Isaac Lab 导入失败

```powershell
# 重新安装
pip uninstall omni.isaac.lab
pip install -e . --no-cache-dir
```

### 3. 端口被占用

```powershell
# 检查端口占用
netstat -ano | findstr :50051

# 杀死占用进程
taskkill /PID <PID> /F
```

### 4. GPU 内存不足

- 降低仿真分辨率
- 减少并行环境数量
- 关闭其他 GPU 应用

## 性能优化

### 1. 启用 TensorRT

```python
# 在 server.py 中
import torch
torch.backends.cudnn.benchmark = True
```

### 2. 调整仿真频率

```python
# 降低频率以提高稳定性
sim_config.dt = 1/60  # 60 Hz
```

### 3. 监控 GPU 使用

```powershell
# 实时监控
nvidia-smi -l 1
```

## 下一步

安装完成后，继续阅读：
- `docs/deployment.md` - 部署架构
- `docs/development.md` - 开发指南
