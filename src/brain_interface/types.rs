use std::fmt;

/// Sensor data received from the simulator
#[derive(Debug, Clone)]
pub struct SensorData {
    pub timestamp: f64,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub orientation: [f32; 4], // xyzw
    pub angular_velocity: [f32; 3],
    pub joint_angles: Vec<f32>,
    pub joint_velocities: Vec<f32>,
    pub foot_contacts: [bool; 2],
    pub is_stable: bool,
}

impl Default for SensorData {
    fn default() -> Self {
        Self {
            timestamp: 0.0,
            position: [0.0; 3],
            velocity: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            joint_angles: vec![0.0; 19],
            joint_velocities: vec![0.0; 19],
            foot_contacts: [false; 2],
            is_stable: true,
        }
    }
}

/// Motor commands sent to the simulator
#[derive(Debug, Clone)]
pub struct MotorCommands {
    pub joint_targets: Vec<f32>,
    pub joint_torques: Option<Vec<f32>>,
    /// Desired body velocity [vx, vy, wz]
    pub desired_velocity: [f32; 3],
}

impl Default for MotorCommands {
    fn default() -> Self {
        Self {
            joint_targets: vec![0.0; 19],
            joint_torques: None,
            desired_velocity: [0.0; 3],
        }
    }
}

impl fmt::Display for MotorCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "vel=({:.2},{:.2},{:.2})",
            self.desired_velocity[0], self.desired_velocity[1], self.desired_velocity[2]
        )
    }
}
