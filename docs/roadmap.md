# Nova 开发路线图

## 定位

```
Becoming（Python）          Nova（Rust）
─────────────────           ──────────────────
大脑：理解世界               身体：执行动作
视觉 + 语言 + 规划           运动控制 + 硬件驱动
输出结构化指令               接收指令 + 适配硬件
```

目标：`任何硬件 + Nova + Becoming = 通用机器人`

---

## 当前进度

### ✅ 已完成
- BrainInterface trait + BaselineGait（fallback 步态）
- RobotIO trait（硬件抽象层）
- Nova ↔ Isaac Lab gRPC 仿真接入
- Nova ↔ 宇树 H1 DDS 真机接入（代码完成）
- 架构文档 / Becoming 设计文档

### 🚧 进行中
- Nova gRPC server（接收 Becoming / Demo 指令）

### 📋 待办
- Demo 客户端（Python，模拟 Becoming 发指令）
- 安全守卫（关节限位、倒地检测、断连保活）
- 端到端仿真联调（Mac Nova → Windows Isaac Lab）

---

## 阶段一：全流程跑通（当前目标）

**目标**：一个完整的闭环跑通，Demo 发指令 → Nova 执行 → 仿真反馈

```
Demo 客户端（Python）
    │ gRPC
    ▼
Nova（Rust，Mac）
    │ gRPC
    ▼
Isaac Lab（Python，Windows）
    │ 物理仿真
    ▼
SensorData 回到 Nova → 回到 Demo
```

### Nova 要做的
- [x] RobotIO trait + Isaac Lab gRPC 接入
- [ ] `proto/nova_control.proto` 定义 Nova 对外接口
- [ ] `src/becoming/mod.rs` gRPC server 监听指令
- [ ] `src/main.rs` 串联控制环路

### Demo 客户端要做的
- [ ] `demo/client.py` Python gRPC 客户端
- [ ] 发送基本指令：前进 / 停止 / 转向
- [ ] 打印机器人状态反馈

### 验收标准
Demo 发"前进 0.5m/s"→ Isaac Lab 里机器人走起来 → 状态数据回到 Demo 打印出来

---

## 阶段二：接入 Becoming

**目标**：用 Becoming 替换 Demo 客户端，接口完全兼容

### Becoming 要做的
- [ ] gRPC client 连接 Nova（复用 Demo 的 proto）
- [ ] 接入 LLM（Claude API）
- [ ] 语音/文字 → 结构化指令 → 发给 Nova

### Nova 要做的
- [ ] 安全守卫（关节限位、倒地检测、断连保活）
- [ ] 状态流式推送给 Becoming

### 验收标准
说"向前走"→ Becoming 解析 → Nova 执行 → 仿真里机器人走

---

## 阶段三：接入视觉

**目标**：Becoming 看到场景，理解后执行

### Becoming 要做的
- [ ] 接入摄像头（仿真虚拟摄像头 / 真机摄像头）
- [ ] VLM 理解场景（LLaVA / PaliGemma）
- [ ] 视觉 + 语言 → 任务规划 → 指令

---

## 阶段四：真机测试

**前提**：阶段一二三在仿真里验证通过

### 要做的
- [ ] Nova 切换到 DDS 真机模式（`--real` flag，代码已写好）
- [ ] 仿真验证过的指令集在真机上测试
- [ ] 安全守卫压测

### 验收标准
Becoming 发指令 → Nova DDS → H1 真机走起来

---

## 阶段五：端到端 VLA（后期）

- [ ] Becoming 训练 VLA 模型
- [ ] 端到端模式：摄像头 + 语言 → 关节角度数组
- [ ] 对齐 LeRobot 接口，复用训练 pipeline
