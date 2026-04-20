use super::{BrainInterface, MotorCommands, SensorData};
use std::f32::consts::PI;

pub struct BaselineGait {
    phase: f32,
    frequency: f32,
    target_velocity: [f32; 3],
}

impl BaselineGait {
    pub fn new() -> Self {
        Self { phase: 0.0, frequency: 1.0, target_velocity: [0.0; 3] }
    }

    pub fn set_target_velocity(&mut self, vx: f32, vy: f32, wz: f32) {
        self.target_velocity = [vx, vy, wz];
    }
}

impl Default for BaselineGait {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainInterface for BaselineGait {
    fn decide(&mut self, _sensors: &SensorData, dt: f32) -> MotorCommands {
        let speed = (self.target_velocity[0].powi(2) + self.target_velocity[1].powi(2)).sqrt();

        if speed < 0.01 {
            return MotorCommands {
                desired_velocity: self.target_velocity,
                ..Default::default()
            };
        }

        self.phase = (self.phase + 2.0 * PI * self.frequency * dt) % (2.0 * PI);
        let s = self.phase.sin();

        // Simple sinusoidal gait: left/right legs in antiphase
        let hip_amp = 0.4_f32;
        let knee_amp = 0.6_f32;
        let arm_amp = 0.3_f32;

        // H1 joint order (19 joints):
        // 0:l_hip_yaw 1:r_hip_yaw 2:torso 3:l_hip_roll 4:r_hip_roll
        // 5:l_shoulder_pitch 6:r_shoulder_pitch 7:l_hip_pitch 8:r_hip_pitch
        // 9:l_shoulder_roll 10:r_shoulder_roll 11:l_knee 12:r_knee
        // 13:l_shoulder_yaw 14:r_shoulder_yaw 15:l_ankle 16:r_ankle
        // 17:l_elbow 18:r_elbow
        let mut targets = vec![0.0f32; 19];
        targets[7] = s * hip_amp;           // l_hip_pitch
        targets[8] = -s * hip_amp;          // r_hip_pitch
        targets[11] = if s > 0.0 { s * knee_amp } else { 0.0 }; // l_knee
        targets[12] = if s < 0.0 { -s * knee_amp } else { 0.0 }; // r_knee
        targets[5] = -s * arm_amp;          // l_shoulder_pitch
        targets[6] = s * arm_amp;           // r_shoulder_pitch

        MotorCommands {
            joint_targets: targets,
            joint_torques: None,
            desired_velocity: self.target_velocity,
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.target_velocity = [0.0; 3];
    }

    fn name(&self) -> &str {
        "BaselineGait"
    }
}
