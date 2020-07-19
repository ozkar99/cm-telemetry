use std::error::Error;
use std::io::Cursor;

use crate::{TelemetryEvent, TelemetryPacket};

extern crate num;
use num::Num;

extern crate binread;
use binread::{BinRead, BinReaderExt};

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

    pub weather: Weather,
    pub track_temperature: i8,
    pub air_temperature: i8,
    pub total_laps: i8,
    pub track_length: i16,
    pub session_type: u8,
    pub track_id: i8,
    pub formula: u8,
    pub session_time_left: u16,
    pub session_duration: u16,
    pub pit_speed_limit: u8,
    pub game_paused: u8,
    pub is_spectating: u8,
    pub spectator_car_index: u8,
    pub sli_pro_native_support: u8,
    pub number_of_marshal_zones: u8,
    pub marshal_zones: [MarshalZone; 21],
    pub safety_car_status: u8,
    pub network_game: u8,
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

#[derive(Debug)]
pub enum Weather {
    Unknown,
    Clear,
    LigthCloud,
    Overcast,
    LightRain,
    HeavyRain,
    Storm,
}

impl Default for Weather {
    fn default() -> Weather {
        Weather::Unknown
    }
}

impl BinRead for Weather {
    type Args = ();
    fn read_options<R: binread::io::Read + binread::io::Seek>(
        reader: &mut R,
        _options: &binread::ReadOptions,
        _args: Self::Args,
    ) -> binread::BinResult<Self> {
        let mut val = [0; core::mem::size_of::<u8>()];
        reader.read_exact(&mut val)?;
        match val[0] {
            0 => Ok(Weather::Clear),
            1 => Ok(Weather::LigthCloud),
            2 => Ok(Weather::Overcast),
            3 => Ok(Weather::LightRain),
            4 => Ok(Weather::HeavyRain),
            5 => Ok(Weather::Storm),
            _ => Ok(Weather::Unknown),
        }
    }
}

#[derive(BinRead, Default, Debug)]
pub struct MarshalZone {
    pub zone_start: f32,
    pub zone_flag: ZoneFlag,
}

#[derive(Debug)]
pub enum ZoneFlag {
    Unknown,
    None,
    Green,
    Blue,
    Yellow,
    Red,
}

impl Default for ZoneFlag {
    fn default() -> ZoneFlag {
        ZoneFlag::Unknown
    }
}

impl BinRead for ZoneFlag {
    type Args = ();
    fn read_options<R: binread::io::Read + binread::io::Seek>(
        reader: &mut R,
        _options: &binread::ReadOptions,
        _args: Self::Args,
    ) -> binread::BinResult<Self> {
        let mut val = [0; core::mem::size_of::<i8>()];
        reader.read_exact(&mut val)?;
        match val[0] {
            0 => Ok(ZoneFlag::None),
            1 => Ok(ZoneFlag::Green),
            2 => Ok(ZoneFlag::Blue),
            3 => Ok(ZoneFlag::Yellow),
            4 => Ok(ZoneFlag::Red),
            _ => Ok(ZoneFlag::Unknown),
        }
    }
}

#[derive(Debug, Default, BinRead)]
pub struct WeatherForecastSample {
    pub session_type: SessionType,
    pub time_offset: u8,
    pub weather: Weather,
    pub track_temperature: i8,
    pub air_temperature: i8,
}

#[derive(Debug)]
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

impl Default for SessionType {
    fn default() -> SessionType {
        SessionType::Unknown
    }
}

impl BinRead for SessionType {
    type Args = ();
    fn read_options<R: binread::io::Read + binread::io::Seek>(
        reader: &mut R,
        _options: &binread::ReadOptions,
        _args: Self::Args,
    ) -> binread::BinResult<Self> {
        let mut val = [0; core::mem::size_of::<u8>()];
        reader.read_exact(&mut val)?;
        match val[0] {
            1 => Ok(SessionType::Practice1),
            2 => Ok(SessionType::Practice2),
            3 => Ok(SessionType::Practice3),
            4 => Ok(SessionType::ShortPractice),
            5 => Ok(SessionType::Qualifier1),
            6 => Ok(SessionType::Qualifier2),
            7 => Ok(SessionType::Qualifier3),
            8 => Ok(SessionType::ShortQualifier),
            9 => Ok(SessionType::OSQ),
            10 => Ok(SessionType::Race),
            11 => Ok(SessionType::Formula2Race),
            12 => Ok(SessionType::TimeTrial),
            _ => Ok(SessionType::Unknown),
        }
    }
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
