//! Walking gait controller
//!
//! Reads WASD keyboard input and drives leg joint targets on the torso
//! via the `JointTargets` component.

use bevy::prelude::*;

use crate::models::humanoid::{JointTargets, Torso};

/// Tracks the current gait phase for stepping animation
#[derive(Component, Default)]
pub struct WalkingGait {
    /// Phase angle (radians) — cycles 0..2π per step
    pub phase: f32,
    /// Current walk speed (mutated by keyboard)
    pub speed: f32,
    /// Which leg leads in the current step (-1=left, 1=right)
    pub lead: f32,
}

const STEP_FREQ: f32 = 4.0;   // radians/sec — how fast steps cycle
const SWING_AMP: f32 = 0.4;   // hip swing amplitude (radians)
const KNEE_BEND: f32 = 0.3;   // knee bend during swing phase (radians)
const LATERAL_AMP: f32 = 0.15; // sideways step amplitude

/// Keyboard-driven walking: updates JointTargets on the torso.
/// PD controller (pd_standing_control) reads these targets each frame.
pub fn walking_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gait_q: Query<(&mut WalkingGait, &mut JointTargets), With<Torso>>,
) {
    let Ok((mut gait, mut targets)) = gait_q.get_single_mut() else {
        return;
    };

    let forward  = keyboard.pressed(KeyCode::KeyW) as i32 as f32;
    let backward = keyboard.pressed(KeyCode::KeyS) as i32 as f32;
    let left     = keyboard.pressed(KeyCode::KeyA) as i32 as f32;
    let right    = keyboard.pressed(KeyCode::KeyD) as i32 as f32;

    let dir = forward - backward;
    let lateral = right - left;

    // No input — decay to standing
    if dir == 0.0 && lateral == 0.0 {
        gait.speed = (gait.speed - 0.5).max(0.0);
        if gait.speed == 0.0 {
            // Restore neutral standing pose
            targets.l_hip_pitch = 0.0;
            targets.r_hip_pitch = 0.0;
            targets.l_knee     = 0.0;
            targets.r_knee     = 0.0;
        }
        return;
    }

    // Ramp up speed
    gait.speed = (gait.speed + 0.8).min(2.0);
    gait.phase += STEP_FREQ * gait.speed * 0.016; // ~60fps assumption

    let phase = gait.phase;
    let lead  = gait.lead;

    // Alternate lead leg each half cycle
    if phase > std::f32::consts::PI {
        gait.lead = -lead;
    }

    let s = (phase * 0.5).sin();
    let l_hip = lead * s * SWING_AMP;
    let r_hip = -lead * s * SWING_AMP;

    // Knee bends during swing (when foot is off ground)
    let l_knee = if l_hip > 0.0 { l_hip.abs() * KNEE_BEND } else { 0.0 };
    let r_knee = if r_hip > 0.0 { r_hip.abs() * KNEE_BEND } else { 0.0 };

    targets.l_hip_pitch = l_hip;
    targets.r_hip_pitch = r_hip;
    targets.l_knee     = l_knee;
    targets.r_knee     = r_knee;

    // Lateral hip compensation (weight shift + leg abduction)
    targets.l_shoulder_pitch = -lateral * LATERAL_AMP * 0.5;
    targets.r_shoulder_pitch =  lateral * LATERAL_AMP * 0.5;
}
