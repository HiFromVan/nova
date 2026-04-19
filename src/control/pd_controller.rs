//! PD Control systems for humanoid joints
//!
//! Architecture:
//!   - `JointTargets` + `JointController` live on the TORSO entity
//!   - `MotorTarget` + `Joint` (which joint type) live on each CHILD body
//!   - The PD system reads torso targets and applies motor commands to each child

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::models::humanoid::{Joint, JointController, JointTargets, MotorTarget};

/// PD controller applied to standing pose.
/// Reads joint targets from the torso and drives each joint motor.
pub fn pd_standing_control(
    mut ctx: WriteDefaultRapierContext,
    torso_q: Query<(&JointTargets, &JointController), With<crate::models::humanoid::Torso>>,
    mut child_q: Query<
        (Entity, &Joint, &mut MotorTarget),
        (With<RigidBody>, Without<crate::models::humanoid::Torso>),
    >,
) {
    let Ok((targets, ctrl)) = torso_q.get_single() else {
        return;
    };

    for (child_ent, joint, mut motor_target) in child_q.iter_mut() {
        let target_angle = match joint {
            Joint::LeftShoulderPitch  => targets.l_shoulder_pitch,
            Joint::LeftElbow          => targets.l_elbow,
            Joint::RightShoulderPitch => targets.r_shoulder_pitch,
            Joint::RightElbow         => targets.r_elbow,
            Joint::LeftHipPitch       => targets.l_hip_pitch,
            Joint::LeftKnee           => targets.l_knee,
            Joint::RightHipPitch      => targets.r_hip_pitch,
            Joint::RightKnee          => targets.r_knee,
            // Head/neck: not PD-controlled here
            Joint::HeadNeck => continue,
        };

        motor_target.0 = target_angle;

        // Get the joint handle and apply PD motor
        let Some(&joint_handle) = ctx.entity2impulse_joint().get(&child_ent) else {
            continue;
        };
        let Some(imp_joint) = ctx.impulse_joints.get_mut(joint_handle) else {
            continue;
        };

        if let Some(revolute) = imp_joint.data.as_revolute_mut() {
            revolute
                .set_motor_model(MotorModel::AccelerationBased)
                .set_motor(target_angle, 0.0, ctrl.kp, ctrl.kd)
                .set_motor_max_force(300.0);
        }
    }
}
