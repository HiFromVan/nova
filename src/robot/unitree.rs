/// Unitree H1 真机后端 — CycloneDDS 直连，500Hz 控制环路
use async_trait::async_trait;
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::NO_STATUS,
        time::{Duration, DurationKind},
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::DdsType,
};
use crate::brain_interface::{SensorData, MotorCommands};
use super::RobotIO;

// ── Unitree H1 DDS 消息类型 ───────────────────────────────────────────────────
// DdsType derive 不支持固定大小数组，全部用 Vec

#[derive(Debug, Clone, DdsType)]
struct LowState {
    #[dust_dds(key)]
    tick: u32,
    // IMU
    imu_quat: Vec<f32>,   // [w, x, y, z]
    imu_gyro: Vec<f32>,   // [x, y, z]
    imu_accel: Vec<f32>,  // [x, y, z]
    imu_rpy: Vec<f32>,    // [roll, pitch, yaw]
    // 19 joints
    joint_q:   Vec<f32>,
    joint_dq:  Vec<f32>,
    joint_tau: Vec<f32>,
    // foot force (4 sensors)
    foot_force: Vec<f32>,
}

#[derive(Debug, Clone, DdsType)]
struct LowCmd {
    #[dust_dds(key)]
    id: u32,
    joint_q:   Vec<f32>,
    joint_dq:  Vec<f32>,
    joint_kp:  Vec<f32>,
    joint_kd:  Vec<f32>,
    joint_tau: Vec<f32>,
}

// ── H1 关节默认站立角度 ────────────────────────────────────────────────────────
const DEFAULT_JOINT_POS: [f32; 19] = [
    0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, -0.4, -0.4,
    0.0, 0.0, 0.8, 0.8,
    0.0, 0.0, -0.4, -0.4,
    0.0, 0.0,
];

const KP: f32 = 80.0;
const KD: f32 = 2.0;

// ── UnitreeRobot ──────────────────────────────────────────────────────────────

pub struct UnitreeRobot {
    reader: dust_dds::subscription::data_reader::DataReader<LowState>,
    writer: dust_dds::publication::data_writer::DataWriter<LowCmd>,
    cmd_id: u32,
}

impl UnitreeRobot {
    pub fn new(domain_id: u32) -> Self {
        let participant = DomainParticipantFactory::get_instance()
            .create_participant(domain_id as i32, QosKind::Default, None, NO_STATUS)
            .expect("DDS participant failed");

        let state_topic = participant
            .create_topic::<LowState>("rt/lowstate", "LowState", QosKind::Default, None, NO_STATUS)
            .unwrap();
        let cmd_topic = participant
            .create_topic::<LowCmd>("rt/lowcmd", "LowCmd", QosKind::Default, None, NO_STATUS)
            .unwrap();

        let reader = participant
            .create_subscriber(QosKind::Default, None, NO_STATUS).unwrap()
            .create_datareader::<LowState>(
                &state_topic,
                QosKind::Specific(DataReaderQos {
                    reliability: ReliabilityQosPolicy {
                        kind: ReliabilityQosPolicyKind::BestEffort,
                        max_blocking_time: DurationKind::Finite(Duration::new(0, 100_000_000)),
                    },
                    durability: DurabilityQosPolicy { kind: DurabilityQosPolicyKind::Volatile },
                    ..Default::default()
                }),
                None, NO_STATUS,
            ).unwrap();

        let writer = participant
            .create_publisher(QosKind::Default, None, NO_STATUS).unwrap()
            .create_datawriter::<LowCmd>(
                &cmd_topic,
                QosKind::Specific(DataWriterQos {
                    reliability: ReliabilityQosPolicy {
                        kind: ReliabilityQosPolicyKind::Reliable,
                        max_blocking_time: DurationKind::Finite(Duration::new(0, 100_000_000)),
                    },
                    ..Default::default()
                }),
                None, NO_STATUS,
            ).unwrap();

        Self { reader, writer, cmd_id: 0 }
    }

    fn read_state(&self) -> Option<LowState> {
        self.reader
            .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .ok()
            .and_then(|s| s.into_iter().next())
            .and_then(|s| s.data().ok())
    }

    fn send_cmd(&mut self, joint_targets: &[f32]) {
        self.cmd_id += 1;
        let cmd = LowCmd {
            id: self.cmd_id,
            joint_q:   joint_targets.to_vec(),
            joint_dq:  vec![0.0; 19],
            joint_kp:  vec![KP; 19],
            joint_kd:  vec![KD; 19],
            joint_tau: vec![0.0; 19],
        };
        let _ = self.writer.write(&cmd, None);
    }

    fn to_sensor(s: &LowState) -> SensorData {
        let quat = &s.imu_quat; // w x y z
        let orientation = if quat.len() >= 4 {
            [quat[1], quat[2], quat[3], quat[0]] // → x y z w
        } else {
            [0.0, 0.0, 0.0, 1.0]
        };
        let rpy = &s.imu_rpy;
        let is_stable = rpy.get(0).map(|r| r.abs() < 0.5).unwrap_or(true)
            && rpy.get(1).map(|p| p.abs() < 0.5).unwrap_or(true);
        let gyro: [f32; 3] = s.imu_gyro.get(..3)
            .and_then(|g| g.try_into().ok()).unwrap_or([0.0; 3]);
        let accel: [f32; 3] = s.imu_accel.get(..3)
            .and_then(|a| a.try_into().ok()).unwrap_or([0.0; 3]);
        SensorData {
            timestamp: 0.0,
            position: [0.0; 3],
            velocity: [accel[0], accel[1], 0.0],
            orientation,
            angular_velocity: gyro,
            joint_angles: s.joint_q.clone(),
            joint_velocities: s.joint_dq.clone(),
            foot_contacts: [
                s.foot_force.get(0).map(|&f| f > 20.0).unwrap_or(false),
                s.foot_force.get(1).map(|&f| f > 20.0).unwrap_or(false),
            ],
            is_stable,
        }
    }
}

#[async_trait]
impl RobotIO for UnitreeRobot {
    async fn reset(&mut self) -> SensorData {
        self.send_cmd(&DEFAULT_JOINT_POS);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        self.read_state().map(|s| Self::to_sensor(&s)).unwrap_or_default()
    }

    async fn step(&mut self, cmd: &MotorCommands) -> SensorData {
        self.send_cmd(&cmd.joint_targets);
        tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
        self.read_state().map(|s| Self::to_sensor(&s)).unwrap_or_default()
    }
}
