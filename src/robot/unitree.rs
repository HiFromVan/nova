/// Unitree H1 真机后端 — CycloneDDS 直连，500Hz 控制环路
use async_trait::async_trait;
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::QosKind,
        qos_policy::{
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
            DurabilityQosPolicy, DurabilityQosPolicyKind,
        },
        status::NO_STATUS,
        time::Duration,
    },
    topic_definition::type_support::DdsType,
};
use crate::brain_interface::{SensorData, MotorCommands};
use super::RobotIO;

// ── Unitree H1 DDS 消息类型（对应 unitree_sdk2 IDL）──────────────────────────

#[derive(Debug, Clone, Default, DdsType)]
struct ImuState {
    pub quaternion: [f32; 4],   // w x y z
    pub gyroscope: [f32; 3],
    pub accelerometer: [f32; 3],
    pub rpy: [f32; 3],
}

#[derive(Debug, Clone, Default, DdsType)]
struct MotorState {
    pub q: f32,
    pub dq: f32,
    pub ddq: f32,
    pub tau_est: f32,
    pub temperature: i8,
}

#[derive(Debug, Clone, Default, DdsType)]
struct LowState {
    pub head: [u8; 2],
    pub level_flag: u8,
    pub frame_reserve: u8,
    pub sn: [u32; 2],
    pub version: [u32; 2],
    pub bandwidth: u16,
    pub imu_state: ImuState,
    pub motor_state: [MotorState; 20],
    pub bms_state: [u8; 32],
    pub foot_force: [i16; 4],
    pub foot_force_est: [i16; 4],
    pub tick: u32,
    pub wireless_remote: [u8; 40],
    pub bit_flag: u32,
    pub adc_remap: f32,
    pub voltage: f32,
    pub reserve: [u32; 2],
    pub crc: u32,
}

#[derive(Debug, Clone, Default, DdsType)]
struct MotorCmd {
    pub mode: u8,
    pub q: f32,
    pub dq: f32,
    pub tau: f32,
    pub kp: f32,
    pub kd: f32,
    pub reserve: [u32; 3],
}

#[derive(Debug, Clone, Default, DdsType)]
struct LowCmd {
    pub head: [u8; 2],
    pub level_flag: u8,
    pub frame_reserve: u8,
    pub sn: [u32; 2],
    pub version: [u32; 2],
    pub bandwidth: u16,
    pub motor_cmd: [MotorCmd; 20],
    pub bms_cmd: [u8; 32],
    pub wireless_remote: [u8; 40],
    pub led: [u8; 12],
    pub fan: [u8; 2],
    pub gpio: u8,
    pub reserve: u32,
    pub crc: u32,
}

// ── H1 关节默认站立角度（policy 训练时的零点参考）────────────────────────────
// 顺序: l_hip_yaw, r_hip_yaw, torso, l_hip_roll, r_hip_roll,
//       l_shoulder_pitch, r_shoulder_pitch, l_hip_pitch, r_hip_pitch,
//       l_shoulder_roll, r_shoulder_roll, l_knee, r_knee,
//       l_shoulder_yaw, r_shoulder_yaw, l_ankle, r_ankle,
//       l_elbow, r_elbow
const DEFAULT_JOINT_POS: [f32; 19] = [
    0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, -0.4, -0.4,
    0.0, 0.0, 0.8, 0.8,
    0.0, 0.0, -0.4, -0.4,
    0.0, 0.0,
];

// PD 增益（参考 unitree_sdk2 H1 示例）
const KP: f32 = 80.0;
const KD: f32 = 2.0;

// ── UnitreeRobot ─────────────────────────────────────────────────────────────

pub struct UnitreeRobot {
    participant: dust_dds::domain::domain_participant::DomainParticipant,
    reader: dust_dds::subscription::data_reader::DataReader<LowState>,
    writer: dust_dds::publication::data_writer::DataWriter<LowCmd>,
    last_state: LowState,
}

impl UnitreeRobot {
    pub fn new(domain_id: u32) -> Self {
        let participant = DomainParticipantFactory::get_instance()
            .create_participant(domain_id as i32, QosKind::Default, None, NO_STATUS)
            .expect("DDS participant failed");

        // 订阅机器人状态
        let sub = participant
            .create_subscriber(QosKind::Default, None, NO_STATUS)
            .unwrap();
        let state_topic = participant
            .create_topic::<LowState>("rt/lowstate", "LowState", QosKind::Default, None, NO_STATUS)
            .unwrap();
        let reader_qos = dust_dds::infrastructure::qos::DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: Duration::new(0, 100_000_000),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::Volatile,
            },
            ..Default::default()
        };
        let reader = sub
            .create_datareader::<LowState>(&state_topic, QosKind::Specific(reader_qos), None, NO_STATUS)
            .unwrap();

        // 发布控制指令
        let pub_ = participant
            .create_publisher(QosKind::Default, None, NO_STATUS)
            .unwrap();
        let cmd_topic = participant
            .create_topic::<LowCmd>("rt/lowcmd", "LowCmd", QosKind::Default, None, NO_STATUS)
            .unwrap();
        let writer_qos = dust_dds::infrastructure::qos::DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: Duration::new(0, 100_000_000),
            },
            ..Default::default()
        };
        let writer = pub_
            .create_datawriter::<LowCmd>(&cmd_topic, QosKind::Specific(writer_qos), None, NO_STATUS)
            .unwrap();

        Self { participant, reader, writer, last_state: LowState::default() }
    }

    fn read_state(&mut self) -> LowState {
        if let Ok(samples) = self.reader.take(1, &[], &[], &[]) {
            if let Some(sample) = samples.into_iter().next() {
                if let Ok(data) = sample.data() {
                    self.last_state = data;
                }
            }
        }
        self.last_state.clone()
    }

    fn send_cmd(&self, joint_targets: &[f32]) {
        let mut cmd = LowCmd::default();
        for i in 0..19.min(joint_targets.len()) {
            cmd.motor_cmd[i] = MotorCmd {
                mode: 1,
                q: joint_targets[i],
                dq: 0.0,
                tau: 0.0,
                kp: KP,
                kd: KD,
                reserve: [0; 3],
            };
        }
        let _ = self.writer.write(&cmd, None);
    }

    fn lowstate_to_sensor(s: &LowState) -> SensorData {
        let imu = &s.imu_state;
        // unitree quaternion 顺序是 w,x,y,z → 转成 x,y,z,w
        let orientation = [imu.quaternion[1], imu.quaternion[2], imu.quaternion[3], imu.quaternion[0]];
        let joint_angles: Vec<f32> = s.motor_state[..19].iter().map(|m| m.q).collect();
        let joint_velocities: Vec<f32> = s.motor_state[..19].iter().map(|m| m.dq).collect();

        // 足部接触：用足力传感器估算（> 20N 认为接触）
        let foot_contacts = [s.foot_force[0] > 20, s.foot_force[1] > 20];

        // 稳定性：用 roll/pitch 判断（< 0.5rad 认为稳定）
        let is_stable = imu.rpy[0].abs() < 0.5 && imu.rpy[1].abs() < 0.5;

        SensorData {
            timestamp: 0.0, // 真机用系统时间
            position: [0.0; 3], // H1 无板载定位，需外部定位系统
            velocity: [imu.accelerometer[0], imu.accelerometer[1], 0.0],
            orientation,
            angular_velocity: imu.gyroscope,
            joint_angles,
            joint_velocities,
            foot_contacts,
            is_stable,
        }
    }
}

#[async_trait]
impl RobotIO for UnitreeRobot {
    async fn reset(&mut self) -> SensorData {
        // 真机 reset：发送站立姿态指令
        self.send_cmd(&DEFAULT_JOINT_POS);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let s = self.read_state();
        Self::lowstate_to_sensor(&s)
    }

    async fn step(&mut self, cmd: &MotorCommands) -> SensorData {
        self.send_cmd(&cmd.joint_targets);
        // 等下一个传感器帧（2ms @ 500Hz）
        tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
        let s = self.read_state();
        Self::lowstate_to_sensor(&s)
    }
}
