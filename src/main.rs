use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod models;
mod simulation;
mod control;
mod brain_interface;

use control::{camera_control, foot_contact_detection, pd_standing_control, walking_control};
use models::humanoid;
use simulation::physics;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_event::<CollisionEvent>()
        .add_systems(Startup, physics::setup_physics)
        .add_systems(Startup, |mut commands: Commands,
                               mut meshes: ResMut<Assets<Mesh>>,
                               mut materials: ResMut<Assets<StandardMaterial>>| {
            println!("=== Spawning humanoid robot ===");
            let entity = humanoid::spawn_humanoid(&mut commands, &mut meshes, &mut materials);
            println!("=== Humanoid spawned with entity: {:?} ===", entity);
        })
        .add_systems(Update, pd_standing_control)
        .add_systems(Update, walking_control)
        .add_systems(Update, foot_contact_detection)
        .add_systems(Update, camera_control)
        .run();
}
