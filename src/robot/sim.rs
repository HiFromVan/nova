/// gRPC 仿真后端 — 仅用于开发验证，不进真机控制环路
use async_trait::async_trait;
use tonic::transport::Channel;
use crate::brain_interface::{SensorData, MotorCommands};
use super::RobotIO;

pub mod proto {
    tonic::include_proto!("nova.simulator");
}
use proto::{simulator_client::SimulatorClient, MotorCommands as ProtoCmd, ResetRequest, Vec3};

pub struct SimRobot {
    client: SimulatorClient<Channel>,
}

impl SimRobot {
    pub async fn connect(addr: &'static str) -> Self {
        let ch = Channel::from_static(addr).connect().await
            .expect("cannot connect to Isaac Lab gRPC server");
        Self { client: SimulatorClient::new(ch) }
    }
}

fn from_proto(s: proto::SensorData) -> SensorData {
    let p = s.position.unwrap_or_default();
    let v = s.velocity.unwrap_or_default();
    let o = s.orientation.unwrap_or_default();
    let a = s.angular_velocity.unwrap_or_default();
    SensorData {
        timestamp: s.timestamp,
        position: [p.x, p.y, p.z],
        velocity: [v.x, v.y, v.z],
        orientation: [o.x, o.y, o.z, o.w],
        angular_velocity: [a.x, a.y, a.z],
        joint_angles: s.joint_angles,
        joint_velocities: s.joint_velocities,
        foot_contacts: [
            s.foot_contacts.first().copied().unwrap_or(false),
            s.foot_contacts.get(1).copied().unwrap_or(false),
        ],
        is_stable: s.is_stable,
    }
}

fn to_proto(cmd: &MotorCommands) -> ProtoCmd {
    ProtoCmd {
        joint_targets: cmd.joint_targets.clone(),
        joint_torques: cmd.joint_torques.clone().unwrap_or_default(),
        desired_velocity: Some(Vec3 {
            x: cmd.desired_velocity[0],
            y: cmd.desired_velocity[1],
            z: cmd.desired_velocity[2],
        }),
    }
}

#[async_trait]
impl RobotIO for SimRobot {
    async fn reset(&mut self) -> SensorData {
        let s = self.client.reset(ResetRequest::default()).await
            .expect("reset failed").into_inner();
        from_proto(s)
    }

    async fn step(&mut self, cmd: &MotorCommands) -> SensorData {
        let s = self.client.step(to_proto(cmd)).await
            .expect("step failed").into_inner();
        from_proto(s)
    }
}
