mod brain_interface;
mod robot;
mod becoming;
mod control;

use brain_interface::{BaselineGait, BrainInterface};
use clap::Parser;
use tokio::sync::watch;

pub mod simulator_proto {
    tonic::include_proto!("nova.simulator");
}

#[derive(Parser)]
struct Args {
    /// 真机模式（DDS），默认仿真模式（gRPC）
    #[arg(long)]
    real: bool,

    /// Isaac Lab gRPC 地址（仿真用）
    #[arg(long, default_value = "http://192.168.50.99:50051")]
    sim_addr: String,

    /// DDS domain id（真机用）
    #[arg(long, default_value_t = 0)]
    domain: u32,

    /// Nova 对外暴露的 gRPC 端口（Becoming / Demo 连接）
    #[arg(long, default_value_t = 50052)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // ── 硬件接口 ──────────────────────────────────────────────────────────────
    let robot: Box<dyn robot::RobotIO> = if args.real {
        panic!("真机模式暂时不可用 - DDS 兼容性问题待修复，请使用仿真模式");
    } else {
        println!("[nova] 模式: 仿真 gRPC ({})", args.sim_addr);
        let addr: &'static str = Box::leak(args.sim_addr.into_boxed_str());
        Box::new(robot::sim::SimRobot::connect(addr).await)
    };

    // ── Brain（Becoming 断连时的 fallback）────────────────────────────────────
    let brain: Box<dyn BrainInterface> = Box::new(BaselineGait::new());

    // ── 共享状态 channel ──────────────────────────────────────────────────────
    let (cmd_tx, cmd_rx) = watch::channel(becoming::BecomingCmd::Stop);
    let (state_tx, state_rx) = watch::channel(brain_interface::SensorData::default());

    // ── 启动 Becoming gRPC server（后台任务）─────────────────────────────────
    let port = args.port;
    tokio::spawn(becoming::serve(cmd_tx, state_rx, port));

    // ── 控制环路（主任务）────────────────────────────────────────────────────
    let hz = if args.real { 500 } else { 50 };
    control::run(robot, brain, cmd_rx, state_tx, hz).await;
}
