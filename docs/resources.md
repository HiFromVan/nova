# Nova 学习资料与参考项目

> 面向软件背景、硬件零基础的人形机器人开发者

---

## 一、入门路径建议

软件背景入门机器人仿真，推荐按这个顺序：

```
机器人学基础概念
    ↓
物理仿真引擎使用（Rapier / MuJoCo）
    ↓
运动控制基础（PD 控制、步态）
    ↓
强化学习 + 机器人（可选，用于训练策略）
    ↓
硬件基础（电机、传感器）
```

---

## 二、视频学习资源

### 2.1 机器人学基础

**Modern Robotics（现代机器人学）**
- 链接：https://www.youtube.com/playlist?list=PLggLP4f-rq02vX0OQQ31plWbLoszKtbMb
- 作者：Kevin Lynch（Northwestern University）
- 内容：运动学、动力学、轨迹规划，配套教材免费
- 适合：系统学习机器人数学基础，有线性代数基础即可

**MIT 6.832 Underactuated Robotics**
- 链接：https://www.youtube.com/channel/UChfUOAhz7ynELF-s_1LPpWg
- 作者：Russ Tedrake（MIT）
- 内容：欠驱动机器人控制，双足步行的核心理论
- 适合：理解为什么双足行走难，以及现代控制方法

**Articulated Robotics（YouTube 频道）**
- 链接：https://www.youtube.com/@ArticulatedRobotics
- 内容：ROS2 + 机器人实践，从仿真到真实硬件
- 适合：快速上手，实践导向，软件背景友好

### 2.2 物理仿真

**MuJoCo Tutorial Series**
- 链接：https://www.youtube.com/playlist?list=PLc7bpbeTIk758Ad3fkSywdxHWpAe4K5i4
- 内容：MuJoCo 仿真器使用，人形机器人建模
- 适合：了解业界主流仿真器的设计思路（对理解 Rapier 有帮助）

**Bevy Game Engine（官方）**
- 链接：https://www.youtube.com/@BevyEngine
- 内容：Bevy ECS 架构、渲染、插件系统
- 适合：深入理解项目使用的引擎

### 2.3 强化学习 + 机器人

**Reinforcement Learning for Robotics（DeepMind）**
- 链接：https://www.youtube.com/watch?v=kopoLzvh5jY
- 内容：RL 在机器人控制中的应用综述

**Legged Robots（ETH Zürich）**
- 链接：https://www.youtube.com/playlist?list=PLZgpos4wVnCYs8UoD0FkSqxAS6NZnea0V
- 内容：腿式机器人控制，包含 ANYmal 四足和双足
- 适合：了解学术界最前沿的腿式机器人控制方法

### 2.4 硬件入门（零基础）

**How Servo Motors Work**
- 链接：https://www.youtube.com/watch?v=1WnGv-DPexc
- 内容：伺服电机工作原理，10 分钟搞懂执行器

**Boston Dynamics 技术讲解（非官方解析）**
- 链接：https://www.youtube.com/watch?v=_sBBaNYex3E
- 内容：Atlas 机器人的控制系统解析
- 适合：建立对"真实机器人控制"的直觉

---

## 三、参考项目

### 3.1 仿真框架

**MuJoCo（DeepMind）**
- 链接：https://github.com/google-deepmind/mujoco
- 语言：C++ / Python
- 说明：业界最主流的机器人仿真器，学术论文标配。Nova 用 Rapier，但 MuJoCo 的设计思路值得参考，尤其是 MJCF 模型格式（XML 描述关节树）

**Isaac Lab（NVIDIA）**
- 链接：https://github.com/isaac-sim/IsaacLab
- 语言：Python
- 说明：GPU 加速并行仿真，强化学习训练人形机器人的工业级框架。Nova 长期目标可参考其 Gym 接口设计

**Genesis**
- 链接：https://github.com/Genesis-Embodied-AI/Genesis
- 语言：Python
- 说明：2024 年底发布，支持超高速并行仿真，社区活跃，代码简洁易读

### 3.2 人形机器人开源项目

**Unitree Robotics H1/G1 SDK**
- 链接：https://github.com/unitreerobotics/unitree_sdk2
- 说明：宇树机器人官方 SDK，真实硬件接口设计参考。brain_interface 的设计可以对标这个

**Berkeley Humanoid**
- 链接：https://github.com/HybridRobotics/berkeley-humanoid
- 说明：伯克利开源的低成本人形机器人，从仿真到硬件完整链路，代码质量高

**Zeroth Bot**
- 链接：https://github.com/zeroth-robotics/zeroth-bot
- 说明：完全开源的人形机器人（包含硬件设计），软件背景友好，社区活跃

**Adam（人形机器人强化学习）**
- 链接：https://github.com/roboverse/adam
- 说明：基于 MuJoCo 的人形机器人 RL 训练框架，Observation/Action 接口设计值得参考

### 3.3 Rust 机器人生态

**k（运动学库）**
- 链接：https://github.com/openrr/k
- 说明：Rust 的机器人运动学库，支持正/逆运动学，Nova 后期 IK 模块可以集成

**openrr**
- 链接：https://github.com/openrr/openrr
- 说明：Rust 机器人框架，类似 ROS 但用 Rust 实现

**bevy_rapier**
- 链接：https://github.com/dimforge/bevy_rapier
- 说明：Nova 直接依赖的库，examples 目录有大量关节、电机的使用示例

### 3.4 控制算法参考

**legged_gym（ETH Zürich）**
- 链接：https://github.com/leggedrobotics/legged_gym
- 说明：腿式机器人 RL 训练的经典框架，Anymal 和人形机器人都用这个训练。Observation/Action 的设计是业界标准参考

**rl_games**
- 链接：https://github.com/Denys88/rl_games
- 说明：配合 Isaac Gym 使用的 RL 训练库，PPO 实现质量高

---

## 四、必读论文（不需要全懂，建立直觉）

| 论文 | 内容 | 为什么读 |
|------|------|---------|
| Learning to Walk in Minutes Using Massively Parallel Deep RL (2021) | 用 GPU 并行仿真快速训练步态 | 理解现代机器人学习的核心思路 |
| Humanoid Locomotion as Next Token Prediction (2024) | 把步态控制当语言模型来做 | 了解 LLM 和机器人的结合方向 |
| Learning Robust Perceptive Locomotion (2022, ETH) | 视觉 + 运动控制 | 理解传感器融合 |

搜索方式：直接 Google 论文标题 + arxiv

---

## 五、工具和文档

| 工具 | 用途 | 链接 |
|------|------|------|
| Rapier 文档 | 关节、刚体、碰撞 API | https://rapier.rs/docs/ |
| Bevy 文档 | ECS、System、插件 | https://bevyengine.org/learn/ |
| Bevy Cheatbook | 快速查阅 Bevy 用法 | https://bevy-cheatbook.github.io/ |
| nalgebra 文档 | 矩阵、四元数运算 | https://nalgebra.org/ |
| URDF 格式说明 | 机器人模型描述格式 | http://wiki.ros.org/urdf |

---

## 六、社区

- **Bevy Discord**：https://discord.gg/bevy — Rapier 集成问题在这里问最快
- **Rapier Discord**：https://discord.gg/vt9DJSW
- **r/robotics**：https://www.reddit.com/r/robotics
- **Humanoid Robot Discord（非官方）**：搜索 "humanoid robotics discord"
