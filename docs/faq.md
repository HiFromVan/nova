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
