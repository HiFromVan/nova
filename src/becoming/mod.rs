use tonic::{transport::Server, Request, Response, Status};
use tokio::sync::watch;
use tokio_stream::StreamExt;

use crate::brain_interface::SensorData;

pub mod proto {
    tonic::include_proto!("nova.control");
}

use proto::{
    nova_control_server::{NovaControl, NovaControlServer},
    ActionVector, RobotState, SemanticCommand, SubscribeRequest,
    semantic_command::Cmd,
    Vec3, Quat,
};

// ── 共享状态 ──────────────────────────────────────────────────────────────────

/// Becoming 下发的最新指令，control loop 消费
#[derive(Clone, Debug)]
pub enum BecomingCmd {
    Move { vx: f32, vy: f32, wz: f32 },
    Stop,
    Act(Vec<f32>),
}

pub type CmdSender   = watch::Sender<BecomingCmd>;
pub type StateSender = watch::Sender<SensorData>;

// ── gRPC service 实现 ─────────────────────────────────────────────────────────

pub struct NovaControlService {
    cmd_tx:   CmdSender,
    state_rx: watch::Receiver<SensorData>,
}

impl NovaControlService {
    pub fn new(cmd_tx: CmdSender, state_rx: watch::Receiver<SensorData>) -> Self {
        Self { cmd_tx, state_rx }
    }
}

fn sensor_to_proto(s: &SensorData) -> RobotState {
    RobotState {
        qpos: s.joint_angles.clone(),
        qvel: s.joint_velocities.clone(),
        position: Some(Vec3 { x: s.position[0], y: s.position[1], z: s.position[2] }),
        orientation: Some(Quat {
            x: s.orientation[0], y: s.orientation[1],
            z: s.orientation[2], w: s.orientation[3],
        }),
        linear_vel: Some(Vec3 { x: s.velocity[0], y: s.velocity[1], z: s.velocity[2] }),
        angular_vel: Some(Vec3 {
            x: s.angular_velocity[0],
            y: s.angular_velocity[1],
            z: s.angular_velocity[2],
        }),
        foot_contacts: s.foot_contacts.to_vec(),
        is_stable: s.is_stable,
        timestamp: s.timestamp,
    }
}

#[tonic::async_trait]
impl NovaControl for NovaControlService {
    async fn command(
        &self,
        req: Request<SemanticCommand>,
    ) -> Result<Response<RobotState>, Status> {
        let cmd = req.into_inner();
        let becoming_cmd = match cmd.cmd {
            Some(Cmd::Move(m))  => BecomingCmd::Move { vx: m.vx, vy: m.vy, wz: m.wz },
            Some(Cmd::Stop(_))  => BecomingCmd::Stop,
            Some(Cmd::Turn(t))  => BecomingCmd::Move { vx: 0.0, vy: 0.0, wz: t.angle },
            None                => BecomingCmd::Stop,
        };
        let _ = self.cmd_tx.send(becoming_cmd);
        let state = sensor_to_proto(&self.state_rx.borrow());
        Ok(Response::new(state))
    }

    async fn act(
        &self,
        req: Request<ActionVector>,
    ) -> Result<Response<RobotState>, Status> {
        let action = req.into_inner();
        if action.joint_angles.len() != 19 {
            return Err(Status::invalid_argument("joint_angles must have 19 elements"));
        }
        let _ = self.cmd_tx.send(BecomingCmd::Act(action.joint_angles));
        let state = sensor_to_proto(&self.state_rx.borrow());
        Ok(Response::new(state))
    }

    type SubscribeStream = std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<RobotState, Status>> + Send>>;

    async fn subscribe(
        &self,
        _req: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let mut state_rx = self.state_rx.clone();
        let (tx, rx) = watch::channel(sensor_to_proto(&state_rx.borrow()));
        tokio::spawn(async move {
            while state_rx.changed().await.is_ok() {
                let _ = tx.send(sensor_to_proto(&state_rx.borrow()));
            }
        });
        let stream = tokio_stream::wrappers::WatchStream::new(rx)

            .map(Ok::<RobotState, Status>);
        Ok(Response::new(Box::pin(stream)))
    }
}

// ── 启动 gRPC server ──────────────────────────────────────────────────────────

pub async fn serve(
    cmd_tx: CmdSender,
    state_rx: watch::Receiver<SensorData>,
    port: u16,
) {
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    let svc = NovaControlServer::new(NovaControlService::new(cmd_tx, state_rx));
    println!("[nova] Becoming gRPC server 监听 :{}", port);
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .expect("gRPC server failed");
}
