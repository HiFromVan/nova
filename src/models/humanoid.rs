use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

// ---------------------------------------------------------------------------
// Config — single source of truth for all body dimensions
// ---------------------------------------------------------------------------

/// All body proportions derived from a single `height` value.
/// Change `height` and everything else scales automatically.
pub struct HumanoidConfig {
    pub height: f32,

    // Segment lengths as fractions of total height
    pub head_frac:      f32,  // head height / total height
    pub torso_frac:     f32,
    pub upper_arm_frac: f32,
    pub forearm_frac:   f32,
    pub thigh_frac:     f32,
    pub shin_frac:      f32,
    pub foot_h_frac:    f32,  // foot height fraction

    // Width/depth ratios relative to height
    pub torso_w_frac:   f32,
    pub shoulder_w_frac: f32, // shoulder width (half, each side from center)
    pub hip_w_frac:     f32,  // hip width (half)
}

impl Default for HumanoidConfig {
    fn default() -> Self {
        Self {
            height: 1.7,
            // Segment fractions (roughly human proportions)
            head_frac:       0.13,
            torso_frac:      0.30,
            upper_arm_frac:  0.17,
            forearm_frac:    0.15,
            thigh_frac:      0.24,
            shin_frac:       0.22,
            foot_h_frac:     0.04,
            // Width fractions
            torso_w_frac:    0.22,
            shoulder_w_frac: 0.22,
            hip_w_frac:      0.12,
        }
    }
}

impl HumanoidConfig {
    pub fn head_h(&self)      -> f32 { self.height * self.head_frac }
    pub fn torso_h(&self)     -> f32 { self.height * self.torso_frac }
    pub fn upper_arm_h(&self) -> f32 { self.height * self.upper_arm_frac }
    pub fn forearm_h(&self)   -> f32 { self.height * self.forearm_frac }
    pub fn thigh_h(&self)     -> f32 { self.height * self.thigh_frac }
    pub fn shin_h(&self)      -> f32 { self.height * self.shin_frac }
    pub fn foot_h(&self)      -> f32 { self.height * self.foot_h_frac }
    pub fn torso_w(&self)     -> f32 { self.height * self.torso_w_frac }
    pub fn shoulder_w(&self)  -> f32 { self.height * self.shoulder_w_frac }
    pub fn hip_w(&self)       -> f32 { self.height * self.hip_w_frac }

    /// Y position of the torso center when standing on the ground (y=0)
    pub fn torso_center_y(&self) -> f32 {
        self.foot_h() + self.shin_h() + self.thigh_h() + self.torso_h() * 0.5
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)] pub struct Humanoid;
#[derive(Component)] pub struct Torso;
#[derive(Component)] pub struct Head;
#[derive(Component)] pub struct Arm;
#[derive(Component)] pub struct Forearm;
#[derive(Component)] pub struct Leg;
#[derive(Component)] pub struct LowerLeg;
#[derive(Component)] pub struct Foot;

/// Set-point angles for each actuated joint (radians)
#[derive(Component, Default)]
pub struct JointTargets {
    pub head_yaw: f32,
    pub head_pitch: f32,
    pub l_shoulder_pitch: f32,
    pub l_elbow: f32,
    pub r_shoulder_pitch: f32,
    pub r_elbow: f32,
    pub l_hip_pitch: f32,
    pub l_knee: f32,
    pub r_hip_pitch: f32,
    pub r_knee: f32,
}

#[derive(Component)]
pub struct JointController {
    pub kp: f32,
    pub kd: f32,
}

impl Default for JointController {
    fn default() -> Self { Self { kp: 80.0, kd: 10.0 } }
}

#[derive(Component)]
pub enum Joint {
    HeadNeck,
    LeftShoulderPitch, LeftElbow,
    RightShoulderPitch, RightElbow,
    LeftHipPitch, LeftKnee,
    RightHipPitch, RightKnee,
}

#[derive(Component)]
pub struct MotorTarget(pub f32);

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

pub fn spawn_humanoid(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    spawn_humanoid_with_config(commands, meshes, materials, &HumanoidConfig::default())
}

pub fn spawn_humanoid_with_config(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    c: &HumanoidConfig,
) -> Entity {
    let mat_body = materials.add(Color::srgb(0.3, 0.5, 0.8));
    let mat_limb = materials.add(Color::srgb(0.2, 0.35, 0.6));

    // Segment half-extents (Rapier cuboid takes half-sizes)
    let torso_hx = c.torso_w() * 0.5;
    let torso_hy = c.torso_h() * 0.5;
    let torso_hz = c.torso_w() * 0.25;

    let head_hs  = c.head_h() * 0.5;

    let arm_hx   = c.torso_w() * 0.12;
    let arm_hy   = c.upper_arm_h() * 0.5;

    let fore_hx  = c.torso_w() * 0.10;
    let fore_hy  = c.forearm_h() * 0.5;

    let thigh_hx = c.hip_w() * 0.55;
    let thigh_hy = c.thigh_h() * 0.5;

    let shin_hx  = c.hip_w() * 0.45;
    let shin_hy  = c.shin_h() * 0.5;

    let foot_hx  = c.hip_w() * 0.5;
    let foot_hy  = c.foot_h() * 0.5;
    let foot_hz  = c.torso_w() * 0.35;

    // ── World-space Y positions of each segment center ────────────────────
    // Built bottom-up from ground (y=0)
    let foot_cy   = foot_hy;
    let shin_cy   = foot_hy * 2.0 + shin_hy;
    let thigh_cy  = foot_hy * 2.0 + c.shin_h() + thigh_hy;
    let torso_cy  = c.torso_center_y();
    let head_cy   = torso_cy + torso_hy + head_hs;
    let arm_cy    = torso_cy + torso_hy * 0.6 - arm_hy;
    let fore_cy   = arm_cy - arm_hy - fore_hy;

    // ── Torso (root) ──────────────────────────────────────────────────────
    let torso = commands.spawn((
        Humanoid, Torso,
        RigidBody::Dynamic,
        Collider::cuboid(torso_hx, torso_hy, torso_hz),
        Mesh3d(meshes.add(Cuboid::new(torso_hx * 2.0, torso_hy * 2.0, torso_hz * 2.0))),
        MeshMaterial3d(mat_body.clone()),
        Transform::from_xyz(0.0, torso_cy, 0.0),
        Visibility::default(),
        JointTargets::default(),
        JointController::default(),
        crate::control::WalkingGait::default(),
    )).id();

    // ── Head ─────────────────────────────────────────────────────────────
    commands.spawn((
        Humanoid, Head,
        RigidBody::Dynamic,
        Collider::cuboid(head_hs, head_hs, head_hs * 0.9),
        Mesh3d(meshes.add(Cuboid::new(head_hs * 2.0, head_hs * 2.0, head_hs * 1.8))),
        MeshMaterial3d(mat_body.clone()),
        Transform::from_xyz(0.0, head_cy, 0.0),
        ImpulseJoint::new(torso, SphericalJointBuilder::new()
            .local_anchor1(Vec3::new(0.0,  torso_hy, 0.0))
            .local_anchor2(Vec3::new(0.0, -head_hs,  0.0))),
        Joint::HeadNeck, MotorTarget(0.0),
    ));

    // ── Left Arm ─────────────────────────────────────────────────────────
    let l_arm = commands.spawn((
        Humanoid, Arm,
        RigidBody::Dynamic,
        Collider::cuboid(arm_hx, arm_hy, arm_hx),
        Mesh3d(meshes.add(Cuboid::new(arm_hx * 2.0, arm_hy * 2.0, arm_hx * 2.0))),
        MeshMaterial3d(mat_limb.clone()),
        Transform::from_xyz(-(c.shoulder_w() + arm_hx), arm_cy, 0.0),
        ImpulseJoint::new(torso, RevoluteJointBuilder::new(Vec3::Z)
            .local_anchor1(Vec3::new(-c.shoulder_w(), torso_hy * 0.6, 0.0))
            .local_anchor2(Vec3::new(0.0, arm_hy, 0.0))
            .limits([-1.5, 1.5])
            .motor_max_force(200.0)),
        Joint::LeftShoulderPitch, MotorTarget(0.0),
    )).id();

    commands.spawn((
        Humanoid, Forearm,
        RigidBody::Dynamic,
        Collider::cuboid(fore_hx, fore_hy, fore_hx),
        Mesh3d(meshes.add(Cuboid::new(fore_hx * 2.0, fore_hy * 2.0, fore_hx * 2.0))),
        MeshMaterial3d(mat_limb.clone()),
        Transform::from_xyz(-(c.shoulder_w() + arm_hx), fore_cy, 0.0),
        ImpulseJoint::new(l_arm, RevoluteJointBuilder::new(Vec3::Z)
            .local_anchor1(Vec3::new(0.0, -arm_hy, 0.0))
            .local_anchor2(Vec3::new(0.0,  fore_hy, 0.0))
            .limits([-2.3, 0.0])
            .motor_max_force(100.0)),
        Joint::LeftElbow, MotorTarget(0.0),
    ));

    // ── Right Arm ────────────────────────────────────────────────────────
    let r_arm = commands.spawn((
        Humanoid, Arm,
        RigidBody::Dynamic,
        Collider::cuboid(arm_hx, arm_hy, arm_hx),
        Mesh3d(meshes.add(Cuboid::new(arm_hx * 2.0, arm_hy * 2.0, arm_hx * 2.0))),
        MeshMaterial3d(mat_limb.clone()),
        Transform::from_xyz(c.shoulder_w() + arm_hx, arm_cy, 0.0),
        ImpulseJoint::new(torso, RevoluteJointBuilder::new(Vec3::Z)
            .local_anchor1(Vec3::new(c.shoulder_w(), torso_hy * 0.6, 0.0))
            .local_anchor2(Vec3::new(0.0, arm_hy, 0.0))
            .limits([-1.5, 1.5])
            .motor_max_force(200.0)),
        Joint::RightShoulderPitch, MotorTarget(0.0),
    )).id();

    commands.spawn((
        Humanoid, Forearm,
        RigidBody::Dynamic,
        Collider::cuboid(fore_hx, fore_hy, fore_hx),
        Mesh3d(meshes.add(Cuboid::new(fore_hx * 2.0, fore_hy * 2.0, fore_hx * 2.0))),
        MeshMaterial3d(mat_limb.clone()),
        Transform::from_xyz(c.shoulder_w() + arm_hx, fore_cy, 0.0),
        ImpulseJoint::new(r_arm, RevoluteJointBuilder::new(Vec3::Z)
            .local_anchor1(Vec3::new(0.0, -arm_hy, 0.0))
            .local_anchor2(Vec3::new(0.0,  fore_hy, 0.0))
            .limits([-2.3, 0.0])
            .motor_max_force(100.0)),
        Joint::RightElbow, MotorTarget(0.0),
    ));

    // ── Left Leg ─────────────────────────────────────────────────────────
    let l_leg = commands.spawn((
        Humanoid, Leg,
        RigidBody::Dynamic,
        Collider::cuboid(thigh_hx, thigh_hy, thigh_hx),
        Mesh3d(meshes.add(Cuboid::new(thigh_hx * 2.0, thigh_hy * 2.0, thigh_hx * 2.0))),
        MeshMaterial3d(mat_body.clone()),
        Transform::from_xyz(-c.hip_w(), thigh_cy, 0.0),
        ImpulseJoint::new(torso, RevoluteJointBuilder::new(Vec3::X)
            .local_anchor1(Vec3::new(-c.hip_w(), -torso_hy, 0.0))
            .local_anchor2(Vec3::new(0.0, thigh_hy, 0.0))
            .limits([-1.5, 1.5])
            .motor_max_force(400.0)),
        Joint::LeftHipPitch, MotorTarget(0.0),
    )).id();

    let l_shin = commands.spawn((
        Humanoid, LowerLeg,
        RigidBody::Dynamic,
        Collider::cuboid(shin_hx, shin_hy, shin_hx),
        Mesh3d(meshes.add(Cuboid::new(shin_hx * 2.0, shin_hy * 2.0, shin_hx * 2.0))),
        MeshMaterial3d(mat_body.clone()),
        Transform::from_xyz(-c.hip_w(), shin_cy, 0.0),
        ImpulseJoint::new(l_leg, RevoluteJointBuilder::new(Vec3::X)
            .local_anchor1(Vec3::new(0.0, -thigh_hy, 0.0))
            .local_anchor2(Vec3::new(0.0,  shin_hy,  0.0))
            .limits([0.0, 2.5])
            .motor_max_force(400.0)),
        Joint::LeftKnee, MotorTarget(0.0),
    )).id();

    commands.spawn((
        Humanoid, Foot,
        RigidBody::Dynamic,
        Collider::cuboid(foot_hx, foot_hy, foot_hz),
        Mesh3d(meshes.add(Cuboid::new(foot_hx * 2.0, foot_hy * 2.0, foot_hz * 2.0))),
        MeshMaterial3d(mat_limb.clone()),
        Transform::from_xyz(-c.hip_w(), foot_cy, foot_hz * 0.3),
        ImpulseJoint::new(l_shin, SphericalJointBuilder::new()
            .local_anchor1(Vec3::new(0.0, -shin_hy, 0.0))
            .local_anchor2(Vec3::new(0.0,  foot_hy, -foot_hz * 0.3))),
        crate::control::FootContact::default(),
    ));

    // ── Right Leg ────────────────────────────────────────────────────────
    let r_leg = commands.spawn((
        Humanoid, Leg,
        RigidBody::Dynamic,
        Collider::cuboid(thigh_hx, thigh_hy, thigh_hx),
        Mesh3d(meshes.add(Cuboid::new(thigh_hx * 2.0, thigh_hy * 2.0, thigh_hx * 2.0))),
        MeshMaterial3d(mat_body.clone()),
        Transform::from_xyz(c.hip_w(), thigh_cy, 0.0),
        ImpulseJoint::new(torso, RevoluteJointBuilder::new(Vec3::X)
            .local_anchor1(Vec3::new(c.hip_w(), -torso_hy, 0.0))
            .local_anchor2(Vec3::new(0.0, thigh_hy, 0.0))
            .limits([-1.5, 1.5])
            .motor_max_force(400.0)),
        Joint::RightHipPitch, MotorTarget(0.0),
    )).id();

    let r_shin = commands.spawn((
        Humanoid, LowerLeg,
        RigidBody::Dynamic,
        Collider::cuboid(shin_hx, shin_hy, shin_hx),
        Mesh3d(meshes.add(Cuboid::new(shin_hx * 2.0, shin_hy * 2.0, shin_hx * 2.0))),
        MeshMaterial3d(mat_body.clone()),
        Transform::from_xyz(c.hip_w(), shin_cy, 0.0),
        ImpulseJoint::new(r_leg, RevoluteJointBuilder::new(Vec3::X)
            .local_anchor1(Vec3::new(0.0, -thigh_hy, 0.0))
            .local_anchor2(Vec3::new(0.0,  shin_hy,  0.0))
            .limits([0.0, 2.5])
            .motor_max_force(400.0)),
        Joint::RightKnee, MotorTarget(0.0),
    )).id();

    commands.spawn((
        Humanoid, Foot,
        RigidBody::Dynamic,
        Collider::cuboid(foot_hx, foot_hy, foot_hz),
        Mesh3d(meshes.add(Cuboid::new(foot_hx * 2.0, foot_hy * 2.0, foot_hz * 2.0))),
        MeshMaterial3d(mat_limb.clone()),
        Transform::from_xyz(c.hip_w(), foot_cy, foot_hz * 0.3),
        ImpulseJoint::new(r_shin, SphericalJointBuilder::new()
            .local_anchor1(Vec3::new(0.0, -shin_hy, 0.0))
            .local_anchor2(Vec3::new(0.0,  foot_hy, -foot_hz * 0.3))),
        crate::control::FootContact::default(),
    ));

    torso
}
