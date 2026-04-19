# Nova 仿真架构设计文档

> 面向软件背景开发者的人形机器人仿真系统设计指南

---

## 一、系统整体架构

```
┌─────────────────────────────────────────┐
│           Brain / AI 决策层              │  ← becoming 项目
│   感知 → 规划 → 决策 → 输出目标姿态       │
├─────────────────────────────────────────┤
│              Brain Interface             │  ← nova/brain_interface
│   Observation（传感器快照）               │
│   Action（关节目标角度/力矩）             │
├─────────────────────────────────────────┤
│              控制层 Control              │  ← nova/control
│   逆运动学 IK → PD 控制器 → 力矩输出     │
├─────────────────────────────────────────┤
│              传感器层 Sensors            │  ← nova/simulation
│   IMU / 关节编码器 / 足底力传感器         │
├─────────────────────────────────────────┤
│           物理仿真层 Physics             │  ← Rapier3D
│   刚体动力学 / 关节约束 / 碰撞检测        │
├─────────────────────────────────────────┤
│              渲染层 Render              │  ← Bevy
│   3D 可视化 / 调试信息 / UI             │
└─────────────────────────────────────────┘
```

**类比软件系统理解：**
- 物理层 = 数据库（存储真实状态）
- 传感器层 = 数据读取 API
- 控制层 = 业务逻辑层
- Brain Interface = 微服务接口
- Brain = 上游服务

---

## 二、机器人物理模型

### 2.1 关节骨架树

人形机器人不是独立的刚体堆叠，而是一棵**关节树**。每个节点是一个刚体，边是关节约束。

```
Pelvis（骨盆，根节点）
├── Spine（脊椎）
│   └── Chest（胸腔）
│       ├── Neck → Head（颈部 → 头部）
│       ├── L_Shoulder → L_Elbow → L_Wrist（左臂）
│       └── R_Shoulder → R_Elbow → R_Wrist（右臂）
├── L_Hip → L_Knee → L_Ankle → L_Foot（左腿）
└── R_Hip → R_Knee → R_Ankle → R_Foot（右腿）
```

**自由度（DOF）参考：**

| 关节 | 类型 | DOF |
|------|------|-----|
| 髋关节 Hip | 球形关节 Spherical | 3 |
| 膝关节 Knee | 转动关节 Revolute | 1 |
| 踝关节 Ankle | 球形关节 Spherical | 2 |
| 肩关节 Shoulder | 球形关节 Spherical | 3 |
| 肘关节 Elbow | 转动关节 Revolute | 1 |
| 腰部 Spine | 球形关节 Spherical | 2 |

典型人形机器人总 DOF：**28~32**

### 2.2 Rapier3D 关节实现

```rust
// 转动关节（膝盖）
let knee_joint = RevoluteJointBuilder::new(Vec3::X)
    .local_anchor1(Vec3::new(0.0, -0.3, 0.0))  // 大腿下端
    .local_anchor2(Vec3::new(0.0,  0.3, 0.0))  // 小腿上端
    .limits([-2.5, 0.0])                         // 角度限位（弧度）
    .motor_position(0.0, 100.0, 10.0)            // 目标角, 刚度Kp, 阻尼Kd
    .build();

commands.spawn(ImpulseJoint::new(thigh_entity, knee_joint));
```

### 2.3 刚体参数

每个身体部件需要设置：
- **质量 mass**：参考真实人体比例（总质量 ~70kg）
- **惯性张量 inertia**：Rapier 可从碰撞体自动计算
- **碰撞体 collider**：用简化几何体（胶囊体/长方体），不用精确网格

---

## 三、执行器模型

### 3.1 软件背景类比

真实机器人关节 ≈ **带反馈的步进电机**：
- 你给它一个目标角度
- 它通过内部 PID 控制器输出力矩
- 有最大力矩限制（电机规格）
- 有响应延迟（电气时间常数）

### 3.2 PD 控制器

仿真中最常用的关节控制方式：

```
力矩 τ = Kp × (目标角度 - 当前角度) - Kd × 当前角速度
```

- **Kp（刚度）**：越大越快到达目标，但容易震荡
- **Kd（阻尼）**：抑制震荡，让运动平滑
- **max_torque**：电机物理上限

```rust
// Rapier 内置 PD 电机
joint.set_motor_position(
    JointAxis::AngX,
    target_angle,   // 目标角度
    stiffness,      // Kp
    damping,        // Kd
);
joint.set_motor_max_force(JointAxis::AngX, max_torque);
```

---

## 四、传感器层

### 4.1 传感器列表

| 传感器 | 真实硬件 | 仿真实现 |
|--------|---------|---------|
| IMU（惯性测量单元） | 陀螺仪 + 加速度计 | 读 Pelvis 的 `Velocity` 组件 |
| 关节编码器 | 磁编码器 | 读关节当前角度 |
| 足底力传感器 | 压力传感器阵列 | Rapier `ContactForce` 事件 |
| 深度相机 | RealSense / ZED | Bevy 渲染到 texture 或射线检测 |

### 4.2 Observation 数据结构

```rust
pub struct Observation {
    // IMU
    pub base_position: Vec3,
    pub base_orientation: Quat,
    pub base_linear_velocity: Vec3,
    pub base_angular_velocity: Vec3,

    // 关节状态（每个关节）
    pub joint_positions: Vec<f32>,    // 当前角度
    pub joint_velocities: Vec<f32>,   // 当前角速度

    // 足底接触
    pub foot_contacts: [bool; 2],     // 左脚/右脚是否着地
    pub foot_forces: [Vec3; 2],       // 接触力
}
```

---

## 五、控制层

### 5.1 控制流水线

```
Brain 输出 Action（目标关节角度）
    ↓
逆运动学 IK（可选，如果 Brain 输出的是末端位置）
    ↓
PD 控制器计算各关节力矩
    ↓
写入 Rapier 关节电机
    ↓
物理引擎步进（固定 dt，通常 1/240s）
    ↓
读取新的传感器数据 → 下一帧
```

### 5.2 Bevy System 调度

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum SimSet {
    SensorRead,      // 读传感器
    BrainTick,       // 调用 Brain
    ControlCompute,  // 计算控制量
    ActuatorApply,   // 写入执行器
}

app.configure_sets(Update, (
    SimSet::SensorRead,
    SimSet::BrainTick,
    SimSet::ControlCompute,
    SimSet::ActuatorApply,
).chain());
```

---

## 六、Brain Interface 设计

### 6.1 接口定义

```rust
pub trait BrainInterface: Send + Sync {
    /// 重置环境，返回初始观测
    fn reset(&mut self) -> Observation;

    /// 执行一步，返回新观测
    fn step(&mut self, obs: &Observation) -> Action;
}

pub struct Action {
    pub joint_targets: Vec<f32>,   // 各关节目标角度
    pub joint_kp: Vec<f32>,        // 各关节刚度（可选，允许 Brain 调整）
    pub joint_kd: Vec<f32>,        // 各关节阻尼（可选）
}
```

### 6.2 两种对接方式

**方案 A：进程内（becoming 是 Rust 库）**
```rust
// 直接 impl BrainInterface
struct BecomingBrain { /* ... */ }
impl BrainInterface for BecomingBrain { /* ... */ }
```

**方案 B：进程间（becoming 是独立进程，支持 Python/其他语言）**
```
Nova 仿真进程  ←→  Unix Socket / gRPC  ←→  becoming 进程
```
接口风格类似 OpenAI Gym：`reset()` / `step(action)` / `render()`

---

## 七、目录结构规划

```
nova/
├── src/
│   ├── main.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── humanoid.rs        # 关节骨架定义
│   │   └── body_params.rs     # 质量、尺寸参数
│   ├── simulation/
│   │   ├── mod.rs
│   │   ├── physics.rs         # 物理世界初始化
│   │   └── sensors.rs         # 传感器读取系统
│   ├── control/
│   │   ├── mod.rs
│   │   ├── pd_controller.rs   # PD 控制器
│   │   ├── ik.rs              # 逆运动学（后期）
│   │   └── gait.rs            # 步态生成（后期）
│   ├── brain_interface/
│   │   ├── mod.rs             # BrainInterface trait
│   │   ├── local.rs           # 进程内对接
│   │   └── ipc.rs             # 进程间通信（后期）
│   └── ui/
│       └── debug_overlay.rs   # 调试信息显示
├── docs/
│   ├── architecture.md        # 本文档
│   └── resources.md           # 学习资料
└── Cargo.toml
```

---

## 八、开发路线图

### Phase 1：物理骨架（当前目标）
- [ ] 用 ImpulseJoint 重写 humanoid，实现完整关节树
- [ ] 各关节正确的角度限位
- [ ] 注册控制 systems，接入键盘调试

### Phase 2：传感器 + 控制
- [ ] 实现 Observation 数据采集
- [ ] PD 控制器让机器人能保持站立姿势
- [ ] 足底接触检测

### Phase 3：Brain Interface
- [ ] 定义 BrainInterface trait
- [ ] 实现简单的规则步态作为 baseline
- [ ] 与 becoming 项目对接

### Phase 4：学习与优化
- [ ] 强化学习训练接口（类 Gym 环境）
- [ ] 并行仿真支持（多个机器人同时训练）
- [ ] 硬件部署接口

---

## 九、硬件基础概念速查

> 给软件背景开发者的最小必要硬件知识

| 概念 | 类比 | 说明 |
|------|------|------|
| 自由度 DOF | 参数维度 | 关节能独立运动的方向数 |
| 力矩 Torque | 写操作的"力度" | 旋转力，单位 N·m |
| 编码器 Encoder | 传感器读数 | 测量关节当前角度 |
| IMU | 陀螺仪 | 测量加速度和角速度 |
| 步态 Gait | 状态机 | 行走时各腿的时序协调 |
| 支撑多边形 | 稳定区域 | 重心投影必须在脚的范围内才不倒 |
| ZMP | 平衡判据 | 零力矩点，判断是否会摔倒 |
| URDF | 配置文件 | 描述机器人结构的 XML 格式 |
