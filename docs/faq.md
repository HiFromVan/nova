# 常见问题

## Q1：宇树 / Tesla 这些公司用什么技术栈？

| 阶段 | 工具 | 语言 |
|------|------|------|
| 仿真训练 | Isaac Lab / MuJoCo | Python |
| 运动策略 | PyTorch / JAX | Python |
| 实时控制 | 自研运行时 | C++ / Rust |
| 硬件驱动 | 厂商 SDK | C++ |

宇树不用 ROS2，用自研的基于 CycloneDDS 的通信中间件。Nova 的方向与此一致。

---

## Q2：Nova 用 Rust 合适吗？需要 ROS2 吗？

Rust 完全合适。实时控制和硬件驱动 Rust 是正确选择，性能接近 C++ 且内存安全。

不需要 ROS2。ROS2 是科研原型工具，量产产品基本都自研或绕过。Nova 自定义接口更轻量可控。

---

## Q3：Isaac Lab 是什么？什么时候需要？

Isaac Lab 是 NVIDIA 的 GPU 加速并行仿真框架，主要用于训练运动策略模型。

**现阶段用途**：仿真验证（Nova 控制逻辑在 Isaac Lab 里跑通后再上真机）。

**不需要用它训练**：宇树 H1 出厂自带运动策略，不需要重新训练。只有开发自定义动作或端到端 VLA 时才需要训练。

---

## Q4：专用机器人和通用机器人架构有什么区别？

**专用（宇树 / 波士顿动力）**
```
RL 训练运动策略 → 部署到机器人 → 执行特定动作
输入：传感器数据  输出：关节角度
```

**通用（Tesla Optimus / Figure）**
```
VLA 模型理解世界 → 规划任务 → 生成动作
输入：摄像头 + 语言  输出：动作序列
```

Nova + Becoming 走的是通用路线：Becoming 是 VLA 大脑，Nova 是硬件适配层。

---

## Q5：Becoming 需要直接接硬件吗？

不需要。职责划分：

```
Becoming 接：摄像头、麦克风、Nova 反馈的机器人状态
Nova 接：电机驱动器、IMU、力传感器

Becoming 永远不直接控制电机
它只告诉 Nova "做什么"，Nova 决定"怎么做"
```

类比：Becoming 是大脑，Nova 是小脑 + 脊髓。走路不需要大脑思考每块肌肉怎么动。

---

## Q6：完整数据流是什么样的？

```
现实世界
  ↓ 摄像头、麦克风
Becoming
  理解场景 → 决定做什么
  ↓ 结构化指令（"走到桌子旁"）
Nova
  指令 → 运动规划 → 关节控制
  ↓ DDS
硬件电机
  ↓ 传感器数据
Nova 采集 → 反馈给 Becoming（闭环）
```

---

## Q7：Becoming 和 Nova 怎么通信？

gRPC（本机 IPC），Nova 是 server，Becoming 是 client。

接口定义在 `proto/nova_control.proto`，支持两种模式：
- **高层模式**：语义指令（move / stop / turn）
- **端到端模式**：关节角度数组（对齐 LeRobot 格式）

---

## Q8：Nova 的商业价值是什么？

```
任何硬件  +  Nova  +  Becoming  =  通用机器人
```

Nova 是硬件和大脑之间的标准化中间层：
- 接入的硬件品牌越多，生态越强
- Becoming 的能力越强，机器人越智能
- 类比：Nova 是 Android（硬件适配），Becoming 是 Google 服务（智能能力）

---

## Q9：BaselineGait 是什么？我需要自己写所有的硬件控制细节吗？

**不需要。** BaselineGait 只是一个临时的 fallback 步态生成器，用于：
- **断连保护**：当 Becoming（大脑）断开连接时，机器人不会失控
- **开发测试**：在 Becoming 还没完成时，能先测试硬件通信
- **最小可行实现**：证明整个架构能跑通

### 真实的控制架构

根据 `docs/architecture.md`，有两种控制模式：

**高层模式（推荐，当前阶段）**
```
Becoming（Python）
  ↓ 发送语义指令：{"move": {"vx": 0.5}}
Nova（Rust）
  ↓ 步态生成（可用现成库）
  ↓ 生成 19 个关节角度
  ↓ 500Hz PD 控制 + 平衡
硬件（H1 电机控制器）
```

你需要做的：
- 不用自己写步态算法
- 可以用：Isaac Lab 的 RL 步态、宇树官方 SDK、开源库（Pinocchio/Drake）

**端到端模式（后期 VLA，最终方案）**
```
Becoming（Python + VLA 模型）
  ↓ 直接输出 19 个关节角度：[0.1, -0.2, ...]
Nova（Rust）
  ↓ 安全检查（关节限位）
  ↓ 500Hz PD 控制
硬件（H1 电机控制器）
```

你需要做的：
- 几乎不用写，只做安全检查
- 对齐 LeRobot 接口，直接用 π0、ACT 等模型

### 建议

**不要在 BaselineGait 上花太多时间**，因为：
1. 它只是临时的 fallback
2. 真实场景会用宇树官方步态或 VLA 模型
3. 现在的重点是**打通整个架构**，而不是优化步态

**推荐路径**：
1. ✅ 阶段一：验证通信链路（已完成）
2. 阶段二：接入 Becoming
3. 用 Isaac Lab 自带的步态控制器或宇树官方步态
4. 最终：端到端 VLA 模式（数据驱动，不用手写步态）

---

## Q10：为什么 Demo 客户端重复发送移动指令没效果？

问题在于指令发送频率太低。原来的实现每 0.5 秒发送一次，导致：
- BaselineGait 的步态相位不连续
- 机器人在指令间隙会停止步态生成
- 容易倒地

**解决方案**：
- 改为每 0.1 秒发送一次指令（10Hz）
- 持续循环发送，而不是发几次就停
- 参考修改后的 `demo/client.py` 的 `run_auto()` 函数

---

## Q11：如何测试完整的闭环？

**测试步骤**：

1. **启动 Isaac Lab 服务器**（Windows）
   ```bash
   # 在 Windows 上运行 Isaac Lab gRPC 服务器
   # 监听 0.0.0.0:50051
   ```

2. **启动 Nova**（Mac 终端 1）
   ```bash
   cd /Users/van/nova
   cargo run
   ```
   应该看到：
   ```
   [nova] 模式: 仿真 gRPC (http://192.168.50.99:50051)
   [nova] Becoming gRPC server 监听 :50052
   [nova] 控制环路启动 @ 50Hz
   ```

3. **运行 Demo 客户端**（Mac 终端 2）
   ```bash
   # 持续走动模式
   python3 demo/client.py --auto

   # 或交互模式（w/s/a/d 控制）
   python3 demo/client.py
   ```

4. **观察结果**
   - Demo 客户端打印机器人状态
   - Nova 终端显示位置和速度
   - Isaac Lab 中机器人应该在移动

**阶段一验收标准**（已完成）：
- ✅ Demo 发"前进 0.5m/s"→ Isaac Lab 里机器人走起来 → 状态数据回到 Demo 打印出来
- ✅ 完整闭环：Demo → Nova → Isaac Lab → 状态反馈
