# Nova — Humanoid Robot Project

Nova is a humanoid robot project built in Rust, designed to work alongside the [Becoming](https://github.com/HiFromVan/becoming) brain system to form a complete autonomous humanoid robot stack.

## Architecture

```
┌─────────────────────────────┐        ┌──────────────────────────┐
│  Isaac Lab  (Windows)        │        │  Nova  (Rust)            │
│                              │        │                          │
│  Physics simulation          │ gRPC   │  BrainInterface trait    │
│  H1 humanoid model (USD)     │◄──────►│  BaselineGait            │
│  RL policy inference         │ :50051 │  High-level decisions    │
│  isaac_env/server.py         │        │  Hardware abstraction    │
└─────────────────────────────┘        └──────────────────────────┘
                                                    ▲
                                                    │
                                        ┌───────────┴──────────┐
                                        │  Becoming  (Mac)     │
                                        │  Decision / planning │
                                        └──────────────────────┘
```

**Isaac Lab** handles physics, rendering, and low-level policy execution on Windows (RTX 3090).  
**Nova** (this repo) is the execution layer — it receives high-level intent and translates it into motor commands.  
**Becoming** is the brain — it decides what to do and sends commands via `BrainInterface`.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core language | Rust |
| ECS / rendering | Bevy 0.15 |
| Local physics | Rapier3D |
| Simulation backend | Isaac Lab (Isaac Sim 4.5, NVIDIA) |
| Brain ↔ Body comms | gRPC (tonic + prost) |
| RL training | RSL-RL 5.x + PPO |

## Project Structure

```
nova/
├── src/
│   ├── brain_interface/   # BrainInterface trait + SensorData / MotorCommands types
│   ├── control/           # Gait, PD controller, keyboard input
│   ├── models/            # Robot model definitions
│   └── simulation/        # Physics bridge
├── examples/
│   └── grpc_client.rs     # gRPC communication test
├── isaac_env/
│   └── server.py          # Isaac Lab gRPC server (Python, runs on Windows)
├── proto/
│   └── simulator.proto    # gRPC service definition
├── build.rs               # Proto compilation
└── TODO.md                # Roadmap and task tracking
```

## Getting Started

### Prerequisites

- Rust 1.95+
- Isaac Lab installed (Windows, see [Isaac Lab setup](https://isaac-sim.github.io/IsaacLab/))
- conda environment `isaaclab` with Isaac Sim 4.5

### Run the gRPC communication test

**1. Start the Isaac Lab simulation server (Windows terminal):**

```bash
cd E:/IsaacLab
conda activate isaaclab
python E:/nova/isaac_env/server.py
```

Wait for `Isaac Lab gRPC Server 已启动 :50051` in the output.

**2. Run the Rust client:**

```bash
cd nova
cargo run --example grpc_client
```

Expected output:
```
→ Reset
← pos=(0.00,1.00,0.00) stable=true
← step 0 pos=(...) joints=[...]
...
gRPC 通信测试完成
```

### Regenerate proto bindings (Python side)

```bash
cd nova
python -m grpc_tools.protoc -I proto --python_out=isaac_env --grpc_python_out=isaac_env proto/simulator.proto
```

## How It Works

1. `server.py` starts Isaac Lab, loads the pretrained H1 locomotion policy, and exposes a gRPC endpoint
2. Rust sends `MotorCommands` (desired velocity or joint targets) via `Step` RPC
3. Isaac Lab runs one simulation step using the RL policy, injecting the desired velocity into the observation
4. Robot state (`SensorData`: position, orientation, joint angles, foot contacts) is returned to Rust
5. Rust `BrainInterface` uses the state to compute the next command

## Relationship to Becoming

- **Nova** (this repo): the robot body — physics, motion control, hardware drivers
- **[Becoming](https://github.com/HiFromVan/becoming)**: the robot brain — perception, planning, decisions

Communication goes through the `BrainInterface` trait. Any brain implementation (rule-based `BaselineGait`, neural network, or Becoming) implements `decide(sensors, dt) -> MotorCommands`.

## Roadmap

See [TODO.md](./TODO.md) for the full task list.

## License

MIT
