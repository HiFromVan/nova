use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod models;
mod simulation;
mod control;
mod brain_interface;

use control::{camera_control, foot_contact_detection, pd_standing_control, keyboard_input_system, brain_control_system};
use models::humanoid;
use simulation::physics;
use brain_interface::BaselineGait;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_event::<CollisionEvent>()
        .add_systems(Startup, physics::setup_physics)
        .add_systems(Startup, setup_robot)
        .add_systems(Update, keyboard_input_system)
        .add_systems(Update, brain_control_system)
        .add_systems(Update, pd_standing_control)
        .add_systems(Update, foot_contact_detection)
        .add_systems(Update, camera_control)
        .run();
}

/// 设置机器人系统
fn setup_robot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("=== Setting up robot ===");

    // 生成机器人
    let robot_entity = humanoid::spawn_humanoid(&mut commands, &mut meshes, &mut materials);
    println!("=== Robot spawned: {:?} ===", robot_entity);

    // 添加 Brain（BaselineGait）
    commands.entity(robot_entity).insert(BaselineGait::new());
    println!("=== BaselineGait attached ===");
}
