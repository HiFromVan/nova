//! Foot contact detection via collision events
//!
//! Uses Rapier collision events to detect when a foot touches the ground.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Marks a foot collider — used to identify foot entities in collision events
#[derive(Component)]
pub struct FootContact {
    /// True when this foot is in contact with the ground
    pub touching_ground: bool,
}

impl Default for FootContact {
    fn default() -> Self {
        Self { touching_ground: false }
    }
}

/// Processes collision events to update `FootContact` state on foot entities.
pub fn foot_contact_detection(
    mut collisions: EventReader<CollisionEvent>,
    mut foot_q: Query<(Entity, &mut FootContact)>,
) {
    for event in collisions.read() {
        match event {
            CollisionEvent::Started(e1, e2, _) => {
                for (foot_ent, mut contact) in foot_q.iter_mut() {
                    if *e1 == foot_ent || *e2 == foot_ent {
                        contact.touching_ground = true;
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                for (foot_ent, mut contact) in foot_q.iter_mut() {
                    if *e1 == foot_ent || *e2 == foot_ent {
                        contact.touching_ground = false;
                    }
                }
            }
        }
    }
}
