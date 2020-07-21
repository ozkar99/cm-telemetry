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
    pub laps: [Lap; 22],
}

#[derive(Debug, Default, BinRead)]
pub struct Lap {
    pub last_lap_time: f32,
    pub current_lap_time: f32,
    pub sector_time_ms: (u16, u16), // sector1, sector2 (no sector3 for some reason)
    pub best_lap_time: f32,
    pub best_lap_number: u8,
    pub best_lap_sector_time: (u16, u16, u16), // sector1, sector2, sector3
    pub best_overall_sector_time: (
        BestOverallSectorTime,
        BestOverallSectorTime,
        BestOverallSectorTime,
    ), // sector1, sector2, sector3
    pub lap_distance: f32,
    pub total_distance: f32,
    pub safety_car_delta: f32,
    pub car_position: u8,
    pub current_lap_number: u8,
    #[br(map = |x: u8| PitStatus::try_from(x).unwrap())]
    pub pit_status: PitStatus,
    #[br(map = |x: u8| Sector::try_from(x).unwrap())]
    pub sector: Sector,
    #[br(map = |x: u8| x > 0)]
    pub current_lap_invalid: bool,
    pub penalties: u8,
    pub grid_position: u8,
    #[br(map = |x: u8| DriverStatus::try_from(x).unwrap())]
    pub driver_status: DriverStatus,
    #[br(map = |x: u8| ResultStatus::try_from(x).unwrap())]
    pub result_status: ResultStatus,
}

impl LapData {
    /// player_data returns the Lap asociated with the player
    pub fn player_data(&self) -> &Lap {
        let player_index = self.header.player_car_index as usize;
        &self.laps[player_index]
    }
}

#[derive(Debug, Default, BinRead)]
pub struct BestLapSectorTime {
    pub sector1: u16,
    pub sector2: u16,
    pub sector3: u16,
}

#[derive(Debug, Default, BinRead)]
pub struct BestOverallSectorTime {
    pub sector_time: u16,
    pub lap_number: u8,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum PitStatus {
    None,
    Pitting,
    InPitArea,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum Sector {
    Sector1,
    Sector2,
    Sector3,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum DriverStatus {
    InGarage,
    FlyingLap,
    InLap,
    OutLap,
    InTrack,
}

#[derive(Debug, BinRead, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum ResultStatus {
    Invalid,
    Inactive,
    Active,
    Finished,
    Disqualified,
    NotClassified,
    Retired,
}

#[derive(Debug)]
pub struct Event {
    pub header: Header,
    pub event_data_details: EventDataDetail,
}

// Event is a bit more complicated since the event_data_details
// depends on some of the packet details, so we cant simply derive and expect it to work
impl binread::BinRead for Event {
    type Args = ();
    fn read_options<R: binread::io::Read + binread::io::Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        args: Self::Args,
    ) -> binread::BinResult<Self> {
        let header = Header::read_options(reader, options, args)?; // re-use Header BinRead default implementation.

        // Read next 4 bytes for event string identification
        let event_code_bytes = <[u8; 4]>::read_options(reader, options, args)?;
        let event_code = std::str::from_utf8(&event_code_bytes).unwrap_or("UNKW");

        let event_data_details = match event_code {
            "SSTA" => EventDataDetail::SessionStarted,
            "SEND" => EventDataDetail::SessionEnded,
            "FTLP" => {
                let idx = <u8>::read_options(reader, options, args)?;
                let time = <f32>::read_options(reader, options, args)?;
                EventDataDetail::FastestLap(idx, time)
            }
            "RTMT" => {
                let idx = <u8>::read_options(reader, options, args)?;
                EventDataDetail::Retirement(idx)
            }
            "DRSE" => EventDataDetail::DRSEnabled,
            "DRSD" => EventDataDetail::DRSDisabled,
            "TMPT" => {
                let idx = <u8>::read_options(reader, options, args)?;
                EventDataDetail::TeamMateInPits(idx)
            }
            "CHQF" => EventDataDetail::ChequeredFlag,
            "RCWN" => {
                let idx = <u8>::read_options(reader, options, args)?;
                EventDataDetail::RaceWinner(idx)
            }
            "PENA" => {
                let detail = PenaltyEventDetail::read_options(reader, options, args)?;
                EventDataDetail::Penalty(detail)
            }
            "SPTP" => {
                let idx = <u8>::read_options(reader, options, args)?;
                let speed = <f32>::read_options(reader, options, args)?;
                EventDataDetail::SpeedTrap(idx, speed)
            }
            _ => EventDataDetail::Unknown,
        };

        Ok(Event {
            header,
            event_data_details,
        })
    }
}

#[derive(Debug)]
pub enum EventDataDetail {
    SessionStarted,
    SessionEnded,
    FastestLap(u8, f32), // time
    Retirement(u8),      // car_index
    DRSEnabled,
    DRSDisabled,
    TeamMateInPits(u8), // car_index
    ChequeredFlag,
    RaceWinner(u8), // car_index
    Penalty(PenaltyEventDetail),
    SpeedTrap(u8, f32), // car_index, speed
    Unknown,            // not part of the spec, added to satisfy match
}

#[derive(Debug, Default, BinRead)]
pub struct PenaltyEventDetail {
    #[br(map = |x: u8| PenaltyType::try_from(x).unwrap())]
    pub penalty_type: PenaltyType,
    #[br(map = |x: u8| InfringementType::try_from(x).unwrap())]
    pub infrigement_type: InfringementType,
    pub vehicle_index: u8,
    pub other_vehicle_index: u8,
    pub time: u8,
    pub lap_number: u8,
    pub places_gained: u8,
}

#[derive(Debug, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum PenaltyType {
    DriveThrough,
    StopGo,
    GridPenalty,
    PenaltyReminder,
    TimePenalty,
    Warning,
    Disqualified,
    RemovedFromFormationLap,
    ParkedTooLongTimer,
    TyreRegulations,
    ThisLapInvalidated,
    ThisAndNextLapInvalidated,
    ThisLapInvalidatedWithNoReason,
    ThisAndNextLapInvalidatedWithNoReason,
    ThisAndPreviousLapInvalidated,
    ThisAndPreviousLapInvalidatedWithNoReason,
    Retired,
    BlackFlagTimer,
}

#[derive(Debug, TryFromPrimitive, EnumDefault)]
#[repr(u8)]
pub enum InfringementType {
    BlockingBySlowDriving,
    BlockingByWrongWayDriving,
    ReversingOffTheStartLine,
    BigCollision,
    SmallCollision,
    CollisionFailedToHandBackPositionSingle,
    CollisionFailedToHandBackPositionMultiple,
    CornerCuttingGainedTime,
    CornerCuttingOvertakeSingle,
    CornerCuttingOvertakeMultiple,
    CrossedPitExitLane,
    IgnoringBlueFlags,
    IgnoringYellowFlags,
    IgnoringDriveThrough,
    TooManyDriveThroughs,
    DriveThroughReminderServeWithinNLaps,
    DriveThroughReminderServeThisLap,
    PitLaneSpeeding,
    ParkedForTooLong,
    IgnoringTyreRegulations,
    TooManyPenalties,
    MultipleWarnings,
    ApproachingDisqualification,
    TyreRegulationsSelectSingle,
    TyreRegulationsSelectMultiple,
    LapInvalidatedCornerCutting,
    LapInvalidatedRunningWide,
    CornerCuttingRanWideGainedTimeMinor,
    CornerCuttingRanWideGainedTimeSignificant,
    CornerCuttingRanWideGainedTimeExtreme,
    LapInvalidatedWallRiding,
    LapInvalidatedFlashbackUsed,
    LapInvalidatedResetToTrack,
    BlockingThePitlane,
    JumpStart,
    SafetyCarToCarCollision,
    SafetyCarIllegalOvertake,
    SafetyCarExceedingAllowedPace,
    VirtualSafetyCarExceedingAllowedPace,
    FormationLapBelowAllowedSpeed,
    RetiredMechanicalFailure,
    RetiredTerminallyDamaged,
    SafetyCarFallingTooFarBack,
    BlackFlagTimer,
    UnservedStopGoPenalty,
    UnservedDriveThroughPenalty,
    EngineComponentChange,
    GearboxChange,
    LeagueGridPenalty,
    RetryPenalty,
    IllegalTimeGain,
    MandatoryPitstop,
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

        let packet_id = packet[5]; // packet_id
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
