use tonic::transport::Channel;

pub mod simulator {
    tonic::include_proto!("nova.simulator");
}

use simulator::{
    simulator_client::SimulatorClient, MotorCommands, ResetRequest, Vec3,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;
    let mut client = SimulatorClient::new(channel);

    // 1. Reset
    println!("→ Reset");
    let resp = client
        .reset(ResetRequest {
            initial_position: None,
            initial_orientation: None,
            randomize: false,
        })
        .await?
        .into_inner();
    println!(
        "← pos=({:.2},{:.2},{:.2}) stable={}",
        resp.position.as_ref().map(|p| p.x).unwrap_or(0.0),
        resp.position.as_ref().map(|p| p.y).unwrap_or(0.0),
        resp.position.as_ref().map(|p| p.z).unwrap_or(0.0),
        resp.is_stable
    );

    // 2. Step x5
    for i in 0..5 {
        let cmd = MotorCommands {
            joint_targets: vec![0.1 * i as f32; 19],
            joint_torques: vec![],
            desired_velocity: Some(Vec3 { x: 0.5, y: 0.0, z: 0.0 }),
        };
        let resp = client.step(cmd).await?.into_inner();
        println!(
            "← step {} pos=({:.2},{:.2},{:.2}) joints={:?}",
            i,
            resp.position.as_ref().map(|p| p.x).unwrap_or(0.0),
            resp.position.as_ref().map(|p| p.y).unwrap_or(0.0),
            resp.position.as_ref().map(|p| p.z).unwrap_or(0.0),
            &resp.joint_angles[..2],
        );
    }

    println!("gRPC 通信测试完成");
    Ok(())
}
