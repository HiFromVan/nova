use crate::brain_interface::{BrainInterface, MotorCommands, SensorData};
use crate::becoming::BecomingCmd;
use tokio::sync::watch;
use tokio::time::{interval, Duration};

pub mod safety;

/// 控制环路：三层频率
///
/// 50Hz  运动规划：从 Becoming 指令生成 MotorCommands
/// 500Hz 控制层：安全检查 + 写入硬件（真机）/ 仿真步进
///
/// 当前实现：50Hz 统一环路（仿真 gRPC 延迟限制），真机切换到 500Hz
pub async fn run(
    mut robot: Box<dyn crate::robot::RobotIO>,
    mut brain: Box<dyn BrainInterface>,
    mut cmd_rx: watch::Receiver<BecomingCmd>,
    state_tx: watch::Sender<SensorData>,
    hz: u64,
) {
    let dt = 1.0 / hz as f32;
    let mut ticker = interval(Duration::from_micros(1_000_000 / hz));

    let mut sensors = robot.reset().await;
    let _ = state_tx.send(sensors.clone());

    let mut target_vel = [0.0f32; 3];
    let mut e2e_action: Option<Vec<f32>> = None;
    let mut step: u64 = 0;

    println!("[nova] 控制环路启动 @ {}Hz", hz);

    loop {
        ticker.tick().await;

        // 读取最新 Becoming 指令（非阻塞）
        if cmd_rx.has_changed().unwrap_or(false) {
            let _ = cmd_rx.changed().await;
            match cmd_rx.borrow().clone() {
                BecomingCmd::Move { vx, vy, wz } => {
                    target_vel = [vx, vy, wz];
                    e2e_action = None;
                }
                BecomingCmd::Stop => {
                    target_vel = [0.0; 3];
                    e2e_action = None;
                }
                BecomingCmd::Act(angles) => {
                    e2e_action = Some(angles);
                }
            }
        }

        // 倒地检测 → 重置
        if !sensors.is_stable {
            println!("[nova] 倒地，重置...");
            sensors = robot.reset().await;
            brain.reset();
            target_vel = [0.0; 3];
            e2e_action = None;
            let _ = state_tx.send(sensors.clone());
            continue;
        }

        // 生成指令
        let cmd = if let Some(ref angles) = e2e_action {
            // 端到端模式：Becoming 直接给关节角度
            let safe = safety::check_joint_limits(angles);
            MotorCommands {
                joint_targets: safe,
                joint_torques: None,
                desired_velocity: target_vel,
            }
        } else {
            // 高层模式：brain 根据速度指令生成步态
            if let Some(bg) = brain.as_any_mut()
                .and_then(|a| a.downcast_mut::<crate::brain_interface::BaselineGait>())
            {
                bg.set_target_velocity(target_vel[0], target_vel[1], target_vel[2]);
            }
            brain.decide(&sensors, dt)
        };

        sensors = robot.step(&cmd).await;
        let _ = state_tx.send(sensors.clone());

        if step % (hz * 5) == 0 {
            println!(
                "[nova] t={:.0}s pos=({:.2},{:.2},{:.2}) vel=({:.2},{:.2},{:.2})",
                step as f32 * dt,
                sensors.position[0], sensors.position[1], sensors.position[2],
                target_vel[0], target_vel[1], target_vel[2],
            );
        }
        step += 1;
    }
}
