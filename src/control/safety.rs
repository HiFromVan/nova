/// H1 关节角度安全限位（单位：rad）
/// 来源：unitree_sdk2 H1 规格
const JOINT_LIMITS: [(f32, f32); 19] = [
    (-0.43,  0.43),  // l_hip_yaw
    (-0.43,  0.43),  // r_hip_yaw
    (-2.35,  2.35),  // torso
    (-0.43,  0.43),  // l_hip_roll
    (-0.43,  0.43),  // r_hip_roll
    (-1.57,  1.57),  // l_shoulder_pitch
    (-1.57,  1.57),  // r_shoulder_pitch
    (-1.57,  1.57),  // l_hip_pitch
    (-1.57,  1.57),  // r_hip_pitch
    (-1.57,  1.57),  // l_shoulder_roll
    (-1.57,  1.57),  // r_shoulder_roll
    ( 0.0,   2.53),  // l_knee
    ( 0.0,   2.53),  // r_knee
    (-1.57,  1.57),  // l_shoulder_yaw
    (-1.57,  1.57),  // r_shoulder_yaw
    (-0.87,  0.52),  // l_ankle
    (-0.87,  0.52),  // r_ankle
    (-1.57,  1.57),  // l_elbow
    (-1.57,  1.57),  // r_elbow
];

/// 最大单步关节变化量（rad），防止突变损坏电机
const MAX_DELTA: f32 = 0.2;

pub fn check_joint_limits(targets: &[f32]) -> Vec<f32> {
    targets
        .iter()
        .enumerate()
        .map(|(i, &q)| {
            if i < JOINT_LIMITS.len() {
                q.clamp(JOINT_LIMITS[i].0, JOINT_LIMITS[i].1)
            } else {
                q
            }
        })
        .collect()
}

pub fn smooth_joints(current: &[f32], targets: &[f32]) -> Vec<f32> {
    current
        .iter()
        .zip(targets.iter())
        .map(|(&cur, &tgt)| {
            let delta = (tgt - cur).clamp(-MAX_DELTA, MAX_DELTA);
            cur + delta
        })
        .collect()
}
