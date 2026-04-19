# Nova 开发路线图

> 从零基础到通用人形机器人的理性分阶段规划

---

## 定位

```
becoming（Python）          Nova（Rust）
─────────────────           ──────────────────
大脑：理解世界               身体：执行动作
视觉 + 语言 + 规划           运动控制 + 硬件驱动
输出结构化指令               接收指令 + 适配硬件
```

Nova 是硬件和大脑之间的标准化中间层，目标是：

```
任何硬件  +  Nova  +  becoming  =  通用机器人
```

---

## 技术栈决策

| 项目 | 语言 | 理由 |
|------|------|------|
| Nova | Rust | 实时控制、硬件驱动，性能和安全都需要 |
| becoming | Python | LLM、视觉模型全是 Python 生态，PyTorch/LangChain |
| 两者通信 | JSON/TCP → gRPC | 先跑通，后期再规范化 |

**不用 ROS2**，自己定义接口标准，更轻量，更可控。

---

## 第一阶段：让仿真里的机器人能动

**目标**：一个能站立、能接收指令移动的仿真机器人

**时间**：1-2 个月

### Nova 要做的
- [x] 重写 `humanoid.rs`，用 ImpulseJoint 把关节树连起来
- [x] 实现 PD 控制器，让机器人能站稳
- [x] 注册 control systems，WASD 键盘控制能走路
- [x] 实现相机控制（鼠标右键旋转）
- [x] 足底接触检测

### becoming 要做的
- 什么都不用做，用键盘代替 becoming 验证控制逻辑

### 这一步的价值
真正理解机器人控制是怎么回事，建立对物理仿真的直觉。

---

## 第二阶段：定义接口，becoming 能发指令

**目标**：becoming 发一条指令，Nova 执行

**时间**：1 个月

### 通信协议（先用最简单的）
```json
{"command": "move", "direction": "forward", "speed": 0.5}
{"command": "turn", "angle": 90}
{"command": "stand_still"}
{"command": "grab", "target": "cup"}
```

### Nova 要做的
- [ ] TCP 服务器监听指令
- [ ] 把 JSON 指令映射到运动执行
- [ ] 把机器人状态序列化反馈给 becoming

### becoming 要做的
- [ ] TCP 客户端发送指令
- [ ] 接收机器人状态

### 这一步的价值
两个项目第一次真正联通，接口契约确立。

---

## 第三阶段：becoming 接入视觉和语言

**目标**：说一句话，机器人执行

**时间**：2-3 个月

### becoming 要做的
- [ ] 接入仿真里的虚拟摄像头画面
- [ ] 接入 LLM（Claude / GPT API）
- [ ] 语言指令 → 结构化命令 → 发给 Nova

### Nova 要做的
- [ ] 虚拟摄像头（Bevy 渲染到 texture）
- [ ] 更丰富的指令集支持

### 这一步的价值
有了可以演示的 demo，语言驱动机器人。

---

## 第四阶段：接真实硬件

**目标**：仿真验证过的控制逻辑跑在真机上

**前提**：前三阶段完成后自然水到渠成

### 设计原则
控制逻辑完全不变，只换 Hardware trait 的实现：

```rust
// 仿真
let hw = SimulatedHardware::new(&rapier_context);

// 真机（宇树 G1）
let hw = UnitreeHardware::new("/dev/can0");
```

### 硬件入门路径
1. 买单关节电机套件（Dynamixel XL430，~$50）练手
2. 组装一条腿（3 个关节）
3. 把仿真里的腿控制代码移植到真实硬件
4. 全身

---

## 接口标准设计（两种模式）

Nova 同时支持高层模式和端到端模式，becoming 根据能力选择：

**高层模式**（becoming 做规划，Nova 做执行）
```
becoming 输出：MoveTo / Grab / Speak 等语义指令
Nova 负责：把语义指令转成运动序列
```

**端到端模式**（becoming 直接输出关节控制）
```
becoming 输出：关节角度数组
Nova 负责：直接写入执行器
```

---

## 与 LeRobot 的对齐策略

HuggingFace LeRobot 的架构和 Nova+becoming 高度对应：

| LeRobot | Nova + becoming |
|---------|-----------------|
| Policy（ACT / Diffusion Policy） | becoming（大脑） |
| Robot env（执行动作） | Nova（身体） |
| Observation space | Nova 状态反馈 |
| Action space | 关节角度数组 |

### 数据格式对齐（阶段二接口设计时落地）

端到端模式的接口直接对齐 LeRobot 约定，becoming 将来可以零改动跑 LeRobot 的 policy：

```json
// 指令（becoming → Nova）
{"action": [0.1, -0.2, 0.5, ...]}

// 状态反馈（Nova → becoming）
{"observation": {"qpos": [...], "qvel": [...]}}
```

### 数据集格式（阶段三收集演示数据时落地）

仿真演示数据直接用 LeRobot 的 dataset 格式（parquet + videos），可以直接喂给 LeRobot 训练流程，不用重新造轮子。

### 仿真环境接口（阶段三）

Nova 仿真层暴露标准 gym 接口给 becoming：

```
reset() → observation
step(action) → observation, reward, done
```

这样 becoming 可以直接复用 LeRobot 的训练代码（无需修改）。

---

## 现在马上要做的一件事

**把仿真里的机器人关节连起来，让它能站稳。**

其他都是后话。
