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

        // 调试：每秒打印一次
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            if COUNTER % 50 == 0 {
                println!("[BaselineGait] phase={:.2} sin={:.2} speed={:.2}", self.phase, s, speed);
            }
        }

        // Simple sinusoidal gait: left/right legs in antiphase
        let hip_amp = 0.3_f32;
        let knee_amp = 0.5_f32;
        let arm_amp = 0.2_f32;

        // Isaac Lab 期望 8 个关节（见 proto/simulator.proto）:
        // [L_hip, L_knee, R_hip, R_knee, L_shoulder, L_elbow, R_shoulder, R_elbow]
        let mut targets = vec![0.0f32; 8];
        targets[0] = s * hip_amp;                                    // L_hip
        targets[1] = if s > 0.0 { s * knee_amp } else { 0.0 };      // L_knee
        targets[2] = -s * hip_amp;                                   // R_hip
        targets[3] = if s < 0.0 { -s * knee_amp } else { 0.0 };     // R_knee
        targets[4] = -s * arm_amp;                                   // L_shoulder
        targets[5] = 0.0;                                            // L_elbow
        targets[6] = s * arm_amp;                                    // R_shoulder
        targets[7] = 0.0;                                            // R_elbow

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

    fn name(&self) -> &str { "BaselineGait" }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
