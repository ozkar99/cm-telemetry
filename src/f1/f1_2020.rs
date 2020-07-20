use std::convert::TryFrom;
use std::error::Error;
use std::io::Cursor;

use crate::{TelemetryEvent, TelemetryPacket};

use binread::{BinRead, BinReaderExt};
use enum_default::EnumDefault;
use num::Num;
use num_enum::TryFromPrimitive;

/// F1_2020 implements the codemasters UDP telemetry protocol for "F1 2020"
/// see: https://forums.codemasters.com/topic/50942-f1-2020-udp-specification/
pub enum F1_2020 {
    Motion(Motion),
    Session(Session),
    LapData(LapData),
    Event(Event),
    Participants(Participants),
    CarSetups(CarSetups),
    CarTelemetry(CarTelemetry),
    CarStatus(CarStatus),
    FinalClassification(FinalClassification),
    LobbyInfo(LobbyInfo),
}

#[derive(Debug, Default, BinRead)]
pub struct Header {
    pub packet_format: u16,
    pub game_major_version: u8,
    pub game_minor_version: u8,
    pub packet_version: u8,
    pub packet_id: u8,
    pub session_uid: u64,
    pub session_time: f32,
    pub frame_identifier: u32,
    pub player_car_index: u8,
    pub secondary_player_car_index: u8,
}

#[derive(Debug, Default, BinRead)]
pub struct Coordinates<T: Num + binread::BinRead<Args = ()>> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Debug, Default, BinRead)]
pub struct WheelValue<T: Num + binread::BinRead<Args = ()>> {
    pub rear_left: T,
    pub rear_right: T,
    pub front_left: T,
    pub front_right: T,
}

#[derive(Debug, BinRead)]
pub struct Motion {
    pub header: Header,
    pub car_motion_data: [CarMotionData; 22],
    pub suspension_position: WheelValue<f32>,
    pub suspension_velocity: WheelValue<f32>,
    pub suspension_acceleration: WheelValue<f32>,
    pub wheel_speed: WheelValue<f32>,
    pub wheel_slip: WheelValue<f32>,
    pub local_velocity: Coordinates<f32>,
    pub angular_velocity: Coordinates<f32>,
    pub angular_acceleration: Coordinates<f32>,
    pub front_wheel_angle: f32,
}

impl Motion {
    /// player_data returns the CarMotionData asociated with the player
    pub fn player_data(&self) -> &CarMotionData {
        let player_index = self.header.player_car_index as usize;
        &self.car_motion_data[player_index]
    }
}

#[derive(Debug, Default, BinRead)]
pub struct CarMotionData {
    pub world_position: Coordinates<f32>,
    pub world_velocity: Coordinates<f32>,
    pub world_forward_dir: Coordinates<i16>,
    pub world_right_dir: Coordinates<i16>,
    pub g_force_lateral: f32,
    pub g_force_longitudinal: f32,
    pub g_force_vertical: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

#[derive(Debug, BinRead)]
pub struct Session {
    pub header: Header,
    #[br(map = |x: u8| Weather::try_from(x).unwrap())]
    pub weather: Weather,
    pub track_temperature: i8,
    pub air_temperature: i8,
    pub total_laps: i8,
    pub track_length: i16,
    #[br(map = |x: u8| SessionType::try_from(x).unwrap())]
    pub session_type: SessionType,
    #[br(map = |x: i8| Track::try_from(x).unwrap())]
    pub track: Track,
    #[br(map = |x: u8| Formula::try_from(x).unwrap())]
    pub formula: Formula,
    pub session_time_left: u16,
    pub session_duration: u16,
    pub pit_speed_limit: u8,
    pub game_paused: u8,
    pub is_spectating: u8,
    pub spectator_car_index: u8,
    pub sli_pro_native_support: u8,
    pub number_of_marshal_zones: u8,
    pub marshal_zones: [MarshalZone; 21],
    #[br(map = |x: u8| SafetyCarStatus::try_from(x).unwrap())]
    pub safety_car_status: SafetyCarStatus,
    #[br(map = |x: u8| x > 0)]
    pub network_game: bool,
    pub number_of_weather_forecast_samples: u8,
    pub weather_forecast_samples: [WeatherForecastSample; 20],
}

impl Session {
    pub fn current_weather_forecast_sample(&self) -> &WeatherForecastSample {
        let mut current_weather_forecast_sample_index =
            self.number_of_weather_forecast_samples as usize;
        if current_weather_forecast_sample_index > 0 {
            current_weather_forecast_sample_index -= 1;
        }
        &self.weather_forecast_samples[current_weather_forecast_sample_index]
    }
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum Weather {
    Clear,
    LigthCloud,
    Overcast,
    LightRain,
    HeavyRain,
    Storm,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(i8)]
pub enum Track {
    Unknown = -1,
    Melbourne,
    PaulRicard,
    Shanghai,
    Sakhir,
    Catalunya,
    Monaco,
    Montreal,
    Silverstone,
    Hockenheim,
    Hungaroring,
    Spa,
    Monza,
    Singapore,
    Suzuka,
    AbuDahbi,
    Texas,
    Brazil,
    Austria,
    Sochi,
    Mexico,
    Baku,
    SakhirShort,
    SilverstoneShort,
    TexasShort,
    SuzukaShort,
    Hanoi,
    Zandvoort,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum Formula {
    F1Modern,
    F1Classic,
    F2,
    F1Generic,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum SafetyCarStatus {
    NoSafetyCar,
    FullSafetyCar,
    VirtualSafetyCar,
}

#[derive(Debug, Default, BinRead)]
pub struct MarshalZone {
    pub zone_start: f32,
    #[br(map = |x: i8| ZoneFlag::try_from(x).unwrap())]
    pub zone_flag: ZoneFlag,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(i8)]
pub enum ZoneFlag {
    Unknown = -1,
    None,
    Green,
    Blue,
    Yellow,
    Red,
}

#[derive(Debug, Default, BinRead)]
pub struct WeatherForecastSample {
    #[br(map = |x: u8| SessionType::try_from(x).unwrap())]
    pub session_type: SessionType,
    pub time_offset: u8,
    #[br(map = |x: u8| Weather::try_from(x).unwrap())]
    pub weather: Weather,
    pub track_temperature: i8,
    pub air_temperature: i8,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum SessionType {
    Unknown,
    Practice1,
    Practice2,
    Practice3,
    ShortPractice,
    Qualifier1,
    Qualifier2,
    Qualifier3,
    ShortQualifier,
    OSQ,
    Race,
    Formula2Race,
    TimeTrial,
}

#[derive(Debug, BinRead)]
pub struct LapData {
    pub header: Header,
}

#[derive(Debug, BinRead)]
pub struct Event {
    pub header: Header,
}

#[derive(Debug, BinRead)]
pub struct Participants {
    pub header: Header,
}

#[derive(Debug, BinRead)]
pub struct CarSetups {
    pub header: Header,
}

#[derive(Debug, BinRead)]
pub struct CarTelemetry {
    pub header: Header,
}

#[derive(Debug, BinRead)]
pub struct CarStatus {
    pub header: Header,
}

#[derive(Debug, BinRead)]
pub struct FinalClassification {
    pub header: Header,
}

#[derive(Debug, BinRead)]
pub struct LobbyInfo {
    pub header: Header,
}

impl TelemetryEvent for F1_2020 {
    fn from_packet(packet: &TelemetryPacket) -> Result<F1_2020, Box<dyn Error>> {
        if packet.len() < 24 {
            return Err(Box::from("Packet is too small to contain a header"));
        }

        let packet_id = packet[5];
        let mut reader = Cursor::new(packet);
        match packet_id {
            0 => {
                let data: Motion = reader.read_le()?;
                Ok(F1_2020::Motion(data))
            }
            1 => {
                let data: Session = reader.read_le()?;
                Ok(F1_2020::Session(data))
            }
            2 => {
                let data: LapData = reader.read_le()?;
                Ok(F1_2020::LapData(data))
            }
            3 => {
                let data: Event = reader.read_le()?;
                Ok(F1_2020::Event(data))
            }
            4 => {
                let data: Participants = reader.read_le()?;
                Ok(F1_2020::Participants(data))
            }
            5 => {
                let data: CarSetups = reader.read_le()?;
                Ok(F1_2020::CarSetups(data))
            }
            6 => {
                let data: CarTelemetry = reader.read_le()?;
                Ok(F1_2020::CarTelemetry(data))
            }
            7 => {
                let data: CarStatus = reader.read_le()?;
                Ok(F1_2020::CarStatus(data))
            }
            8 => {
                let data: FinalClassification = reader.read_le()?;
                Ok(F1_2020::FinalClassification(data))
            }
            9 => {
                let data: LobbyInfo = reader.read_le()?;
                Ok(F1_2020::LobbyInfo(data))
            }
            id => Err(Box::from(format!("Unknown packet type: {}", id))),
        }
    }
}
