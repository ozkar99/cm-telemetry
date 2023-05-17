use std::convert::TryFrom;
use std::error::Error;
use std::io::Cursor;

use crate::{f1::macros::*, f1::util::*, TelemetryEvent, TelemetryPacket};

use binread::{BinRead, BinReaderExt};
use bitflags::bitflags;
use num_enum::TryFromPrimitive;

/// F1_2022 implements the codemasters UDP telemetry protocol for "F1 22"
/// See: https://answers.ea.com/t5/General-Discussion/F1-22-UDP-Specification/td-p/11551274
/// Or: https://answers.ea.com/t5/General-Discussion/F1-22-UDP-Specification/td-p/11551274?attachment-id=657933

pub enum F1_2022 {
    Motion(Motion),
    Session(Session),
    LapData(LapData),
    Event(Event),
    Participants(Participants),
    CarSetup(CarSetup),
    CarTelemetry(CarTelemetry),
    CarStatus(CarStatus),
    FinalClassification(FinalClassification),
    LobbyInfo(LobbyInfo),
    CarDamage(CarDamage),
    SessionHistory(SessionHistory),
}

// HEADER
#[derive(Debug, Default, BinRead)]
pub struct Header {
    pub packet_format: u16,     // 2022
    pub game_major_version: u8, // Game major version - "X.00"
    pub game_minor_version: u8, // Game minor version - "1.XX"
    pub packet_version: u8,     // Version of this packet type, all start from 1
    pub packet_id: u8,          // Identifier for the packet type, see below
    pub session_uid: u64,       // Unique identifier for the session
    pub session_time: f32,      // Session timestamp
    pub frame_identifier: u32,  // Identifier for the frame the data was retrieved on
    pub player_car_index: u8,   // Index of player's car in the array
    pub secondary_player_car_index: u8, // Index of secondary player's car in the array (splitscreen)
                                        // 255 if no second player
}

// MOTION
#[derive(Debug, BinRead)]
pub struct Motion {
    pub header: Header,

    #[br(count = 22)]
    pub car_motion_data: Vec<CarMotionData>, // Data for all cars on track (22)

    // Extra player car ONLY data
    pub suspension_position: WheelValue<f32>, // Note: All wheel arrays have the following order:
    pub suspension_velocity: WheelValue<f32>, // RL, RR, FL, FR
    pub suspension_acceleration: WheelValue<f32>, // RL, RR, FL, FR
    pub wheel_speed: WheelValue<f32>,         // Speed of each wheel
    pub wheel_slip: WheelValue<f32>,          // Slip ratio for each wheel
    pub local_velocity: Coordinates<f32>,     // Velocity in local space
    pub angular_velocity: Coordinates<f32>,   // Angular velocity
    pub angular_acceleration: Coordinates<f32>, // Angular acceleration
    pub front_wheel_angle: f32,               // Current front wheels angle in radians
}

player_data!(Motion, CarMotionData, car_motion_data);

#[derive(Debug, Default, BinRead)]
pub struct CarMotionData {
    pub world_position: Coordinates<f32>,    // World space position
    pub world_velocity: Coordinates<f32>,    // Velocity in world space
    pub world_forward_dir: Coordinates<i16>, // World space forward direction (normalised)
    pub world_right_dir: Coordinates<i16>,   // World space right direction (normalised)
    pub g_force_lateral: f32,                // Lateral G-Force component
    pub g_force_longitudinal: f32,           // Longitudinal G-Force component
    pub g_force_vertical: f32,               // Vertical G-Force component
    pub yaw: f32,                            // Yaw angle in radians
    pub pitch: f32,                          // Pitch angle in radians
    pub roll: f32,                           // Roll angle in radians
}

// SESSION
#[derive(Debug, BinRead)]
pub struct Session {
    pub header: Header,
    pub weather: Weather, // Weather - 0 = clear, 1 = light cloud, 2 = overcast
    // 3 = light rain, 4 = heavy rain, 5 = storm
    pub track_temperature: i8,     // Track temp. in degrees celsius
    pub air_temperature: i8,       // Air temp. in degrees celsius
    pub total_laps: u8,            // Total number of laps in this race
    pub track_length: u16,         // Track length in metres
    pub session_type: SessionType, // 0 = unknown, 1 = P1, 2 = P2, 3 = P3, 4 = Short P
    // 5 = Q1, 6 = Q2, 7 = Q3, 8 = Short Q, 9 = OSQ
    // 10 = R, 11 = R2, 12 = R3, 13 = Time Trial
    pub track: Track,     // -1 for unknown, see appendix
    pub formula: Formula, // Formula, 0 = F1 Modern, 1 = F1 Classic, 2 = F2,
    // 3 = F1 Generic, 4 = Beta, 5 = Supercars
    // 6 = Esports, 7 = F2 2021
    pub session_time_left: u16,      // Time left in session in seconds
    pub session_duration: u16,       // Session duration in seconds
    pub pit_speed_limit: u8,         // Pit speed limit in kilometres per hour
    pub game_paused: u8,             // Whether the game is paused – network game only
    pub is_spectating: u8,           // Whether the player is spectating
    pub spectator_car_index: u8,     // Index of the car being spectated
    pub sli_pro_native_support: u8,  // SLI Pro support, 0 = inactive, 1 = active
    pub number_of_marshal_zones: u8, // Number of marshal zones to follow
    #[br(count = 21)]
    pub marshal_zones: Vec<MarshalZone>, // List of marshal zones – max 21
    pub safety_car_status: SafetyCarStatus, // 0 = no safety car, 1 = full
    // 2 = virtual, 3 = formation lap
    #[br(map = |x: u8| x > 0)]
    pub network_game: bool, // 0 = offline, 1 = online
    pub number_of_weather_forecast_samples: u8, // Number of weather samples to follow
    #[br(count = 56)] //Array does not work due to size being > 32
    pub weather_forecast_samples: Vec<WeatherForecastSample>, // Array of weather forecast samples
    pub forecast_accuracy: ForecastAccuracy,    // 0 = Perfect, 1 = Approximate
    pub ai_difficulty: u8,                      // AI Difficulty rating – 0-110
    pub season_link_identifier: u32,            // Identifier for season - persists across saves
    pub weekend_link_identifier: u32,           // Identifier for weekend - persists across saves
    pub session_link_identifier: u32,           // Identifier for session - persists across saves
    pub pit_stop_window_ideal_lap: u8,          // Ideal lap to pit on for current strategy (player)
    pub pit_stop_window_latest_lap: u8, // Latest lap to pit on for current strategy (player)
    pub pit_stop_rejoin_position: u8,   // Predicted position to rejoin at (player)
    #[br(map = |x: u8| x > 0)]
    pub steering_assist: bool, // 0 = off, 1 = on
    pub braking_assist: BrakingAssist,  // 0 = off, 1 = low, 2 = medium, 3 = high
    pub gearbox_assist: GearboxAssist,  // 1 = manual, 2 = manual & suggested gear, 3 = auto
    #[br(map = |x: u8| x > 0)]
    pub pit_assist: bool, // 0 = off, 1 = on
    #[br(map = |x: u8| x > 0)]
    pub pit_release_assist: bool, // 0 = off, 1 = on
    #[br(map = |x: u8| x > 0)]
    pub ers_assist: bool, // 0 = off, 1 = on
    #[br(map = |x: u8| x > 0)]
    pub drs_assist: bool, // 0 = off, 1 = on
    pub dynamic_racing_line: RacingLine, // 0 = off, 1 = corners only, 2 = full
    pub dynamic_racing_line_type: RacingLineType, // 0 = 2D, 1 = 3D
    pub game_mode: GameMode,            // Game mode id - see appendix
    pub rule_set: RuleSet,              // Ruleset - see appendix
    pub time_of_day: u32,               // Local time of day - minutes since midnight
    pub session_length: SessionLength,  // 0 = None, 2 = Very Short, 3 = Short, 4 = Medium
                                        // 5 = Medium Long, 6 = Long, 7 = Full
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Weather {
    #[default]
    Clear,
    LigthCloud,
    Overcast,
    LightRain,
    HeavyRain,
    Storm,
    Unknown = 255,
}

binread_enum!(Weather, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum SessionType {
    #[default]
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
    R2,
    R3,
    TimeTrial,
}

binread_enum!(SessionType, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(i8)]
pub enum Track {
    #[default]
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
    Imola,
    Portimao,
    Jeddah,
    Miami,
}

binread_enum!(Track, i8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Formula {
    #[default]
    F1Modern,
    F1Classic,
    F2,
    F1Generic,
    Beta,
    Supercars,
    Esports,
    Unknown = 255,
}

binread_enum!(Formula, u8);

#[derive(Debug, Default, BinRead)]
pub struct MarshalZone {
    pub zone_start: f32, // Fraction (0..1) of way through the lap the marshal zone starts
    pub zone_flag: ZoneFlag, // -1 = invalid/unknown, 0 = none, 1 = green, 2 = blue, 3 = yellow, 4 = red
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(i8)]
pub enum ZoneFlag {
    #[default]
    Unknown = -1,
    None,
    Green,
    Blue,
    Yellow,
    Red,
}

binread_enum!(ZoneFlag, i8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum SafetyCarStatus {
    #[default]
    NoSafetyCar,
    FullSafetyCar,
    VirtualSafetyCar,
    FormationLap,
    Unknown = 255,
}

binread_enum!(SafetyCarStatus, u8);

#[derive(Debug, Default, BinRead)]
pub struct WeatherForecastSample {
    pub session_type: SessionType, // 0 = unknown, 1 = P1, 2 = P2, 3 = P3, 4 = Short P, 5 = Q1
    // 6 = Q2, 7 = Q3, 8 = Short Q, 9 = OSQ, 10 = R, 11 = R2
    // 12 = R3, 13 = Time Trial
    pub time_offset: u8,  // Time in minutes the forecast is for
    pub weather: Weather, // Weather - 0 = clear, 1 = light cloud, 2 = overcast
    // 3 = light rain, 4 = heavy rain, 5 = storm
    pub track_temperature: i8, // Track temp. in degrees Celsius
    pub track_temperature_change: WeatherTemperatureTrend, // Track temp. change – 0 = up, 1 = down, 2 = no change
    pub air_temperature: i8,                               // Air temp. in degrees celsius
    pub air_temperature_change: WeatherTemperatureTrend, // Air temp. change – 0 = up, 1 = down, 2 = no change
    pub rain_percentage: u8,                             // Rain percentage (0-100)
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(i8)]
pub enum WeatherTemperatureTrend {
    #[default]
    Unknown = -1,
    Up,
    Down,
    NoChange,
}

binread_enum!(WeatherTemperatureTrend, i8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum ForecastAccuracy {
    #[default]
    Perfect,
    Approximate,
    Unknown = 255,
}

binread_enum!(ForecastAccuracy, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum BrakingAssist {
    #[default]
    Off,
    Low,
    Medium,
    High,
    Unknown = 255,
}

binread_enum!(BrakingAssist, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum GearboxAssist {
    #[default]
    Manual = 1,
    ManualAndSuggest,
    Auto,
    Unknown = 255,
}

binread_enum!(GearboxAssist, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum RacingLine {
    #[default]
    Off,
    CornersOnly,
    Full,
    Unknown = 255,
}

binread_enum!(RacingLine, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum RacingLineType {
    #[default]
    TwoD,
    ThreeD,
    Unknown = 255,
}

binread_enum!(RacingLineType, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum GameMode {
    #[default]
    EventMode,
    GrandPrix = 3,
    TimeTrial = 5,
    Splitscreen,
    OnlineCustom,
    OnlineLeague,
    CareerInvitational = 11,
    ChampionshipInvitational,
    Championship,
    OnlineChampionship,
    OnlineWeeklyEvent,
    Career22 = 19,
    Career22Online,
    Benchmark = 127,
    Unknown = 255,
}

binread_enum!(GameMode, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum RuleSet {
    #[default]
    PracticeAndQualifying,
    Race,
    TimeTrial,
    TimeAttack = 4,
    CheckpointChallenge = 6,
    Autocross = 8,
    Drift,
    AverageSpeedZone,
    RivalDuel,
    Unknown = 255,
}

binread_enum!(RuleSet, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum SessionLength {
    #[default]
    None,
    VeryShort = 2,
    Short,
    Medium,
    MediumLong,
    Long,
    Full,
    Unknown = 255,
}

binread_enum!(SessionLength, u8);

// LAP
#[derive(Debug, BinRead)]
pub struct LapData {
    pub header: Header,
    #[br(count = 22)]
    pub laps: Vec<Lap>, // Lap data for all cars on track
    pub time_trial_pb_car_idx: u8, // Index of Personal Best car in time trial (255 if invalid)
    pub time_trial_rival_car_idx: u8, // Index of Rival car in time trial (255 if invalid)
}

player_data!(LapData, Lap, laps);

#[derive(Debug, Default, BinRead)]
pub struct Lap {
    pub last_lap_time_ms: u32,      // Last lap time in milliseconds
    pub current_lap_time_ms: u32,   // Current time around the lap in milliseconds
    pub sector_time_ms: (u16, u16), // sector1, sector2 (no sector3 for some reason)
    pub lap_distance: f32,          // Distance vehicle is around current lap in metres – could
    // be negative if line hasn’t been crossed yet
    pub total_distance: f32, // Total distance travelled in session in metres – could
    // be negative if line hasn’t been crossed yet
    pub safety_car_delta: f32,  // Delta in seconds for safety car
    pub car_position: u8,       // Car race position
    pub current_lap_number: u8, // Current lap number
    pub pit_status: PitStatus,  // 0 = none, 1 = pitting, 2 = in pit area
    pub num_pit_stops: u8,      // Number of pit stops taken in this race
    pub sector: Sector,         // 0 = sector1, 1 = sector2, 2 = sector3
    #[br(map = |x: u8| x > 0)]
    pub current_lap_invalid: bool, // Current lap invalid - 0 = valid, 1 = invalid
    pub penalties: u8,          // Accumulated time penalties in seconds to be added
    pub warnings: u8,           // Accumulated number of warnings issued
    pub num_unserved_drive_through_penalties: u8, // Num drive through pens left to serve
    pub num_unserved_stop_go_penalties: u8, // Num stop go pens left to serve
    pub grid_position: u8,      // Grid position the vehicle started the race in
    pub driver_status: DriverStatus, // Status of driver - 0 = in garage, 1 = flying lap
    // 2 = in lap, 3 = out lap, 4 = on track
    pub result_status: ResultStatus, // Result status - 0 = invalid, 1 = inactive, 2 = active
    // 3 = finished, 4 = didnotfinish, 5 = disqualified
    // 6 = not classified, 7 = retired
    #[br(map = |x: u8| x > 0)]
    pub pit_lane_timer_active: bool, // Pit lane timing, 0 = inactive, 1 = active
    pub pit_lane_time_in_lane_ms: u16, // If active, the current time spent in the pit lane in ms
    pub pit_stop_timer_ms: u16,        // Time of the actual pit stop in ms
    pub pit_stop_should_serve_penalty: u8, // Whether the car should serve a penalty at this stop
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum PitStatus {
    #[default]
    None,
    Pitting,
    InPitArea,
    Unknown = 255,
}

binread_enum!(PitStatus, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Sector {
    Sector1,
    Sector2,
    Sector3,
    #[default]
    Unknown = 255,
}

binread_enum!(Sector, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum DriverStatus {
    InGarage,
    FlyingLap,
    InLap,
    OutLap,
    OnTrack,
    #[default]
    Unknown = 255,
}

binread_enum!(DriverStatus, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum ResultStatus {
    Invalid,
    Inactive,
    Active,
    Finished,
    DidNotFinished,
    Disqualified,
    NotClassified,
    Retired,
    #[default]
    Unknown = 255,
}

binread_enum!(ResultStatus, u8);

// EVENT
#[derive(Debug)]
pub struct Event {
    pub header: Header,
    pub event_data_details: EventDataDetail, // Event details - should be interpreted differently
                                             // for each type
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
                let detail = SpeedTrapDetail::read_options(reader, options, args)?;
                EventDataDetail::SpeedTrap(detail)
            }
            "STLG" => {
                let num_lights = <u8>::read_options(reader, options, args)?;
                EventDataDetail::StartLights(num_lights)
            }
            "LGOT" => EventDataDetail::LightsOut,
            "DTSV" => {
                let idx = <u8>::read_options(reader, options, args)?;
                EventDataDetail::DriveThroughServed(idx)
            }
            "SGSV" => {
                let idx = <u8>::read_options(reader, options, args)?;
                EventDataDetail::StopGoServed(idx)
            }
            "FLBK" => {
                let flashback_frame_identifier = <u32>::read_options(reader, options, args)?;
                let flashback_session_time = <f32>::read_options(reader, options, args)?;
                EventDataDetail::Flashback(flashback_frame_identifier, flashback_session_time)
            }
            "BUTN" => {
                let button_status =
                    ButtonFlags::from_bits(<u32>::read_options(reader, options, args)?)
                        .unwrap_or_default();
                EventDataDetail::ButtonStatus(button_status)
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
    FastestLap(u8, f32), // vehicleIdx; Vehicle index of car achieving fastest lap
    // lapTime; Lap time is in seconds
    Retirement(u8), // vehicleIdx; Vehicle index of car retiring
    DRSEnabled,
    DRSDisabled,
    TeamMateInPits(u8), // vehicleIdx; Vehicle index of team mate
    ChequeredFlag,
    RaceWinner(u8), // vehicleIdx; Vehicle index of the race winner
    Penalty(PenaltyEventDetail),
    SpeedTrap(SpeedTrapDetail),
    StartLights(u8), // numLights; Number of lights showing
    LightsOut,
    DriveThroughServed(u8), // vehicleIdx; Vehicle index of the vehicle serving drive through
    StopGoServed(u8),       // vehicleIdx; Vehicle index of the vehicle serving stop go
    Flashback(u32, f32),    // flashbackFrameIdentifier; Frame identifier flashed back to
    // flashbackSessionTime; Session time flashed back to
    ButtonStatus(ButtonFlags), // buttonStatus; Bit flags specifying which buttons are being pressed
    // currently - see appendices
    Unknown, // not part of the spec, added to satisfy match
}

bitflags! {
    #[derive(Debug)]
    pub struct ButtonFlags: u32 {
        const CROSS_OR_A        = 0x00000001;
        const TRIANGLE_OR_Y     = 0x00000002;
        const CIRCLE_OR_B       = 0x00000004;
        const SQUARE_OR_X       = 0x00000008;
        const D_PAD_LEFT        = 0x00000010;
        const D_PAD_RIGHT       = 0x00000020;
        const D_PAD_UP          = 0x00000040;
        const D_PAD_DOWN        = 0x00000080;
        const OPTIONS_OR_MENU   = 0x00000100;
        const L1_OR_LB          = 0x00000200;
        const R1_OR_RB          = 0x00000400;
        const L2_OR_LT          = 0x00000800;
        const R2_OR_RT          = 0x00001000;
        const LEFT_STICK_CLICK  = 0x00002000;
        const RIGHT_STICK_CLICK = 0x00004000;
        const RIGHT_STICK_LEFT  = 0x00008000;
        const RIGHT_STICK_RIGHT = 0x00010000;
        const RIGHT_STICK_UP    = 0x00020000;
        const RIGHT_STICK_DOWN  = 0x00040000;
        const SPECIAL           = 0x00080000;
        const UDP_ACTION_1      = 0x00100000;
        const UDP_ACTION_2      = 0x00200000;
        const UDP_ACTION_3      = 0x00400000;
        const UDP_ACTION_4      = 0x00800000;
        const UDP_ACTION_5      = 0x01000000;
        const UDP_ACTION_6      = 0x02000000;
        const UDP_ACTION_7      = 0x04000000;
        const UDP_ACTION_8      = 0x08000000;
        const UDP_ACTION_9      = 0x10000000;
        const UDP_ACTION_10     = 0x20000000;
        const UDP_ACTION_11     = 0x40000000;
        const UDP_ACTION_12     = 0x80000000;
    }
}

impl Default for ButtonFlags {
    fn default() -> Self {
        ButtonFlags::empty()
    }
}

#[derive(Debug, Default, BinRead)]
pub struct PenaltyEventDetail {
    pub penalty_type: PenaltyType,          // Penalty type – see Appendices
    pub infrigement_type: InfringementType, // Infringement type – see Appendices
    pub vehicle_index: u8,                  // Vehicle index of the car the penalty is applied to
    pub other_vehicle_index: u8,            // Vehicle index of the other car involved
    pub time: u8,                           // Time gained, or time spent doing action in seconds
    pub lap_number: u8,                     // Lap the penalty occurred on
    pub places_gained: u8,                  // Number of places gained by this
}

#[derive(Debug, Default, TryFromPrimitive)]
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
    #[default]
    Unknown = 255,
}

binread_enum!(PenaltyType, u8);

#[derive(Debug, Default, TryFromPrimitive)]
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
    FormationLapParking,
    RetiredMechanicalFailure,
    RetiredTerminallyDamaged,
    SafetyCarFallingTooFarBack,
    BlackFlagTimer,
    UnservedStopGoPenalty,
    UnservedDriveThroughPenalty,
    EngineComponentChange,
    GearboxChange,
    ParcFermeChange,
    LeagueGridPenalty,
    RetryPenalty,
    IllegalTimeGain,
    MandatoryPitstop,
    AttributeAssigned,
    #[default]
    Unknown = 255,
}

binread_enum!(InfringementType, u8);

#[derive(Debug, Default, BinRead)]
pub struct SpeedTrapDetail {
    pub vehicle_index: u8, // Vehicle index of the vehicle triggering speed trap
    pub speed: f32,        // Top speed achieved in kilometres per hour
    #[br(map = |x: u8| x > 0)]
    pub is_overall_fastest_in_session: bool, // Overall fastest speed in session = 1, otherwise 0
    #[br(map = |x: u8| x > 0)]
    pub is_driver_fastest_in_session: bool, // Fastest speed for driver in session = 1, otherwise 0
    pub fastest_vehicle_index_in_session: u8, // Vehicle index of the vehicle that is the fastest
    // in this session
    pub fastest_speed_in_session: f32, // Speed of the vehicle that is the fastest
                                       // in this session
}

// PARTICIPANTS
#[derive(Debug, BinRead)]
pub struct Participants {
    pub header: Header,
    pub num_active_cars: u8, // Number of active cars in the data – should match number of
    // cars on HUD
    #[br(count = 22)]
    pub participants_data: Vec<ParticipantsData>,
}

player_data!(Participants, ParticipantsData, participants_data);

#[derive(Debug, Default, BinRead)]
pub struct ParticipantsData {
    #[br(map = |x: u8| x > 0)]
    pub ai_controlled: bool, // Whether the vehicle is AI (1) or Human (0) controlled
    pub driver: Driver, // Driver id - see appendix, 255 if network human
    pub network_id: u8, // Network id – unique identifier for network players
    pub team: Team,     // Team id - see appendix
    #[br(map = |x: u8| x > 1)]
    pub my_team: bool, // My team flag – 1 = My Team, 0 = otherwise
    pub race_number: u8, // Race number of the car
    pub nationality: Nationality, // Nationality of the driver
    #[br(parse_with = participant_name_parser)]
    pub name: String, // Name of participant in UTF-8 format – null terminated
    // Will be truncated with … (U+2026) if too long
    #[br(map = |x: u8| x > 1)]
    pub your_telemetry_public: bool, // The player's UDP setting, 0 = restricted, 1 = public
}

fn participant_name_parser<R: binread::io::Read + binread::io::Seek>(
    reader: &mut R,
    _: &binread::ReadOptions,
    _: (),
) -> binread::BinResult<String> {
    let mut bytes: [u8; 48] = [0; 48]; // names for participants are 48 bytes wide
    reader.read_exact(&mut bytes)?;

    let driver_name = std::str::from_utf8(&bytes)
        .unwrap_or("UNKW")
        .trim_matches(char::from(0)); // trim any additional null-bytes

    Ok(String::from(driver_name))
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Driver {
    CarlosSainz,
    DaniilKvyat,
    DanielRicciardo,
    FernandoAlonso,
    FelipeMassa,
    KimiRaikkonen = 6,
    LewisHamilton,
    MaxVerstappen = 9,
    NicoHulkenburg,
    KevinMagnussen,
    RomainGrosjean,
    SebastianVettel,
    SergioPerez,
    ValtteriBottas,
    EstebanOcon = 17,
    LanceStroll = 19,
    ArronBarnes,
    MartinGiles,
    AlexMurray,
    LucasRoth,
    IgorCorreia,
    SophieLevasseur,
    JonasSchiffer,
    AlainForest,
    JayLetourneau,
    EstoSaari,
    YasarAtiyeh,
    CallistoCalabresi,
    NaotaIzum,
    HowardClarke,
    WilheimKaufmann,
    MarieLaursen,
    FlavioNieves,
    PeterBelousov,
    KlimekMichalski,
    SantiagoMoreno,
    BenjaminCoppens,
    NoahVisser,
    GertWaldmuller,
    JulianQuesada,
    DanielJones,
    ArtemMarkelov,
    TadasukeMakino,
    SeanGelael,
    NyckDeVries,
    JackAitken,
    GeorgeRussell,
    MaximilianGunther,
    NireiFukuzumi,
    LucaGhiotto,
    LandoNorris,
    SergioSetteCamara,
    LouisDeletraz,
    AntonioFuoco,
    CharlesLeclerc,
    PierreGasly,
    AlexanderAlbon = 62,
    NicholasLatifi,
    DorianBoccolacci,
    NikoKari,
    RobertoMerhi,
    ArjunMaini,
    AlessioLorandi,
    RubenMeijer,
    RashidNair,
    JackTremblay,
    DevonButler,
    LukasWeber,
    AntonioGiovinazzi,
    RobertKubica,
    AlainProst,
    AyrtonSenna,
    NobuharuMatsushita,
    NikitaMazepin,
    GuanyaZhou,
    MickSchumacher,
    CallumIlott,
    JuanManuelCorrea,
    JordanKing,
    MahaveerRaghunathan,
    TatianaCalderon,
    AnthoineHubert,
    GuilianoAlesi,
    RalphBoschung,
    MichaelSchumacher,
    DanTicktum,
    MarcusArmstrong,
    ChristianLundgaard,
    YukiTsunoda,
    JehanDaruvala,
    GulhermeSamaia,
    PedroPiquet,
    FelipeDrugovich,
    RobertSchwartzman,
    RoyNissany,
    MarinoSato,
    AidanJackson,
    CasperAkkerman,
    JensonButton = 109,
    DavidCoulthard,
    NicoRosberg,
    OscarPiastri,
    LiamLawson,
    JuriVips,
    TheoPourchaire,
    RichardVerschoor,
    LirimZendeli,
    DavidBeckmann,
    AlessioDeledda = 121,
    BentViscaal,
    EnzoFittipaldi,
    MarkWebber = 125,
    JacquesVilleneuve,
    JakeHughes,
    FrederikVesti,
    OlliCaldwell,
    LoganSargeant,
    CemBolukbasi,
    AyumaIwasa,
    ClementNovolak,
    DennisHauger,
    CalanWilliams,
    JackDoohan,
    AmauryCordeel,
    MikaHakkinen,

    #[default]
    Unknown,
    Human = 255, // Used for time trial "ghost" drivers that appear randomly
}

binread_enum!(Driver, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Team {
    Mercedes,
    Ferrari,
    RedBullRacing,
    Williams,
    AstonMartin,
    Alpine,
    AlphaTauri,
    Haas,
    McLaren,
    AlfaRomeo,
    Mercedes2020 = 85,
    Ferrari2020,
    RedBull2020,
    Williams2020,
    RacingPoint2020,
    Renault2020,
    AlphaTauri2020,
    Haas2020,
    McLaren2020,
    AlfaRomeo2020,
    AstonMartinDB11V12,
    AstonMartinVantageF1Edition,
    AstonMartinVantageSafetyCar,
    FerrariF8Tributo,
    FerrariRoma,
    McLaren720S,
    McLarenArtura,
    MercedesAMGGTBlackSeriesSafetyCar,
    MercedesAMGGTRPro,
    F1CustomTeam,
    Prema2021,
    UniVirtuosi2021,
    Carlin2021,
    Hitech2021,
    ArtGP2021,
    MPMotorsport2021,
    Charouz2021,
    Dams2021,
    Campos2021,
    BWT2021,
    Trident2021,
    MercedesAMGGTBlackSeries,
    Prema2022,
    Virtuosi2022,
    Carlin2022,
    Hitech2022,
    ArtGP2022,
    MPMotorsport2022,
    Charouz2022,
    Dams2022,
    Campos2022,
    VanAmersfoortRacing2022,
    Trident2022,
    #[default]
    Unknown = 255,
}

binread_enum!(Team, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Nationality {
    #[default]
    Unknown,
    American,
    Argentinean,
    Australian,
    Austrian,
    Azerbaijani,
    Bahraini,
    Belgian,
    Bolivian,
    Brazilian,
    British,
    Bulgarian,
    Cameroonian,
    Canadian,
    Chilean,
    Chinese,
    Colombian,
    CostaRican,
    Croatian,
    Cypriot,
    Czech,
    Danish,
    Dutch,
    Ecuadorian,
    English,
    Emirian,
    Estonian,
    Finnish,
    French,
    German,
    Ghanaian,
    Greek,
    Guatemalan,
    Honduran,
    HongKonger,
    Hungarian,
    Icelander,
    Indian,
    Indonesian,
    Irish,
    Israeli,
    Italian,
    Jamaican,
    Japanese,
    Jordanian,
    Kuwaiti,
    Latvian,
    Lebanese,
    Lithuanian,
    Luxembourger,
    Malaysian,
    Maltese,
    Mexican,
    Monegasque,
    NewZealander,
    Nicaraguan,
    NorthernIrish,
    Norwegian,
    Omani,
    Pakistani,
    Panamanian,
    Paraguayan,
    Peruvian,
    Polish,
    Portuguese,
    Qatari,
    Romanian,
    Russian,
    Salvadoran,
    Saudi,
    Scottish,
    Serbian,
    Singaporean,
    Slovakian,
    Slovenian,
    SouthKorean,
    SouthAfrican,
    Spanish,
    Swedish,
    Swiss,
    Thai,
    Turkish,
    Uruguayan,
    Ukrainian,
    Venezuelan,
    Barbadian,
    Welsh,
    Vietnamese,
}

binread_enum!(Nationality, u8);

// CAR SETUP
#[derive(Debug, BinRead)]
pub struct CarSetup {
    pub header: Header,
    #[br(count = 22)]
    pub car_setup_data: Vec<CarSetupData>,
}

player_data!(CarSetup, CarSetupData, car_setup_data);

#[derive(Debug, Default, BinRead)]
pub struct CarSetupData {
    pub wing: FrontRearValue<u8>,              // Wing aero
    pub on_throttle: u8,                       // Differential adjustment on throttle (percentage)
    pub off_throttle: u8,                      // Differential adjustment off throttle (percentage)
    pub camber: FrontRearValue<f32>,           // Camber angle (suspension geometry)
    pub toe: FrontRearValue<f32>,              // Toe angle (suspension geometry)
    pub suspension: FrontRearValue<u8>,        // Suspension
    pub anti_roll_bar: FrontRearValue<u8>,     // Anti-roll bar
    pub suspension_height: FrontRearValue<u8>, // Ride height
    pub brake_pressure: u8,                    // Brake pressure (percentage)
    pub brake_bias: u8,                        // Brake bias (percentage)
    pub type_pressure: WheelValue<f32>,        // Tyre pressure (PSI)
    pub ballast: u8,                           // Ballast
    pub fuel_load: f32,                        // Fuel load
}

// CAR TELEMETRY
#[derive(Debug, BinRead)]
pub struct CarTelemetry {
    pub header: Header,
    #[br(count = 22)]
    pub car_telemetry_data: Vec<CarTelemetryData>,
    pub mfd_panel: MFDPanel, // Index of MFD panel open - 255 = MFD closed
    // Single player, race – 0 = Car setup, 1 = Pits
    // 2 = Damage, 3 =  Engine, 4 = Temperatures
    // May vary depending on game mode
    pub mfd_panel_secondary_player: MFDPanel, // See above
    #[br(map = |x: i8| if x == 0 { Gear::Unknown } else { Gear::try_from(x).unwrap() })]
    pub suggested_gear: Gear, // Suggested gear for the player (1-8)
                                              // 0 if no gear suggested
}

player_data!(CarTelemetry, CarTelemetryData, car_telemetry_data);

#[derive(Debug, Default, BinRead)]
pub struct CarTelemetryData {
    pub speed: u16,    // Speed of car in kilometres per hour
    pub throttle: f32, // Amount of throttle applied (0.0 to 1.0)
    pub steer: f32,    // Steering (-1.0 (full lock left) to 1.0 (full lock right))
    pub brake: f32,    // Amount of brake applied (0.0 to 1.0)
    pub clutch: u8,    // Amount of clutch applied (0 to 100)
    #[br(map = |x: i8| Gear::try_from(x).unwrap())]
    pub gear: Gear, // Gear selected (1-8, N=0, R=-1)
    pub engine_rpm: u16, // Engine RPM
    #[br(map = |x: u8| x > 0)]
    pub drs: bool, // 0 = off, 1 = on
    pub rev_lights_percent: u8, // Rev lights indicator (percentage)
    pub rev_lights_bit_value: u16, // Rev lights (bit 0 = leftmost LED, bit 14 = rightmost LED)
    pub brake_temp: WheelValue<u16>, // Brakes temperature (celsius)
    pub tyres_surface_temp: WheelValue<u8>, // Tyres surface temperature (celsius)
    pub tyres_inner_temp: WheelValue<u8>, // Tyres inner temperature (celsius)
    pub engine_temp: u16, // Engine temperature (celsius)
    pub tyres_pressure: WheelValue<f32>, // Tyres pressure (PSI)
    #[br(parse_with = surface_type_parser)]
    pub surface_type: WheelValue<Surface>, // Driving surface, see appendices
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(i8)]
pub enum Gear {
    Reverse = -1,
    Neutral,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eigth,
    #[default]
    Unknown = 127,
}

binread_enum!(Gear, i8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Surface {
    Tarmac,
    RumbleStrip,
    Concrete,
    Rock,
    Gravel,
    Mud,
    Sand,
    Grass,
    Water,
    Cobblestone,
    Metal,
    Ridged,
    #[default]
    Unknown = 255,
}

binread_enum!(Surface, u8);

fn surface_type_parser<R: binread::io::Read + binread::io::Seek>(
    reader: &mut R,
    _: &binread::ReadOptions,
    _: (),
) -> binread::BinResult<WheelValue<Surface>> {
    let mut bytes: [u8; 4] = [0; 4];
    reader.read_exact(&mut bytes)?;

    Ok(WheelValue::<Surface> {
        rear_left: Surface::try_from(bytes[0]).unwrap_or(Surface::Unknown),
        rear_right: Surface::try_from(bytes[1]).unwrap_or(Surface::Unknown),
        front_left: Surface::try_from(bytes[2]).unwrap_or(Surface::Unknown),
        front_right: Surface::try_from(bytes[3]).unwrap_or(Surface::Unknown),
    })
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum MFDPanel {
    CarSetup,
    Pits,
    Damage,
    Engine,
    Temperatures,
    #[default]
    Unknown = 128,
    Closed = 255,
}

binread_enum!(MFDPanel, u8);

// CAR STATUS

#[derive(Debug, BinRead)]
pub struct CarStatus {
    pub header: Header,
    #[br(count = 22)]
    pub car_status_data: Vec<CarStatusData>,
}

player_data!(CarStatus, CarStatusData, car_status_data);

#[derive(Debug, Default, BinRead)]
pub struct CarStatusData {
    pub traction_control: u8, // Traction control - 0 = off, 1 = medium, 2 = full
    #[br(map = |x: u8| x > 0)]
    pub anti_lock_brakes: bool, // 0 (off) - 1 (on)
    pub fuel_mix: FuelMix,    // Fuel mix - 0 = lean, 1 = standard, 2 = rich, 3 = max
    pub front_brake_bias: u8, // Front brake bias (percentage)
    #[br(map = |x: u8| x > 0)]
    pub pit_limiter_status: bool, // Pit limiter status - 0 = off, 1 = on
    pub fuel_in_tank: f32,    // Current fuel mass
    pub fuel_capacity: f32,   // Fuel capacity
    pub fuel_remaining_laps: f32, // Fuel remaining in terms of laps (value on MFD)
    pub max_rpm: u16,         // Cars max RPM, point of rev limiter
    pub idle_rpm: u16,        // Cars idle RPM
    pub max_gears: u8,        // Maximum number of gears
    #[br(map = |x: u8| x > 0)]
    pub drs_allowed: bool, // 0 = not allowed, 1 = allowed
    #[br(map = |x: u16| if x > 0 { DRSActivationDistance::Distance(x) } else { DRSActivationDistance::NotAvailable })]
    pub drs_activation_distance: DRSActivationDistance, // 0 = DRS not available, non-zero - DRS will be available
    // in [X] metres
    pub tyres_compound: TyreCompound, // F1 Modern - 16 = C5, 17 = C4, 18 = C3, 19 = C2, 20 = C1
    // 7 = inter, 8 = wet
    // F1 Classic - 9 = dry, 10 = wet
    // F2 – 11 = super soft, 12 = soft, 13 = medium, 14 = hard
    // 15 = wet
    pub tyres_visual: TyreVisual, // F1 visual (can be different from actual compound)
    // 16 = soft, 17 = medium, 18 = hard, 7 = inter, 8 = wet
    // F1 Classic – same as above
    // F2 ‘19, 15 = wet, 19 – super soft, 20 = soft
    // 21 = medium , 22 = hard
    pub tyres_ages_lap: u8,        // Age in laps of the current set of tyres
    pub vehicle_fia_flag: FiaFlag, // -1 = invalid/unknown, 0 = none, 1 = green
    // 2 = blue, 3 = yellow, 4 = red
    pub ers_data: ERS,      // ERS Data
    pub network_paused: u8, // Whether the car is paused in a network game
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum FuelMix {
    Lean,
    Standard,
    Rich,
    Max,
    #[default]
    Unknown,
}

binread_enum!(FuelMix, u8);

#[derive(Debug, Default)]
#[repr(u16)]
pub enum DRSActivationDistance {
    #[default]
    NotAvailable,
    Distance(u16),
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum TyreCompound {
    Inter = 7,
    Wet,
    F1ClassicDry,
    F1ClassicWet,
    F2SuperSoft,
    F2Soft,
    F2Medium,
    F2Hard,
    F2Wet,
    C5,
    C4,
    C3,
    C2,
    C1,
    #[default]
    Unknown,
}

binread_enum!(TyreCompound, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum TyreVisual {
    Inter = 7,
    Wet,
    ClassicDry = 9,
    ClassicWet,
    F2Wet = 15,
    Soft,
    Medium,
    Hard,
    F2SuperSoft,
    F2Soft,
    F2Medium,
    F2Hard,
    #[default]
    Unknown = 255,
}

binread_enum!(TyreVisual, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(i8)]
pub enum FiaFlag {
    #[default]
    Unknown = -1,
    None,
    Green,
    Blue,
    Yellow,
    Red,
}

binread_enum!(FiaFlag, i8);

#[derive(Debug, Default, BinRead)]
pub struct ERS {
    pub stored_energy: f32,         // ERS energy store in Joules
    pub deploy_mode: ERSDeployMode, // ERS deployment mode, 0 = none, 1 = medium
    // 2 = hotlap, 3 = overtake
    pub harvested_this_lap_mguk: f32, // ERS energy harvested this lap by MGU-K
    pub harvested_this_lap_mguh: f32, // ERS energy harvested this lap by MGU-H
    pub deployed_this_lap: f32,       // ERS energy deployed this lap
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum ERSDeployMode {
    None,
    Medium,
    Hotlap,
    Overtake,
    #[default]
    Unknown = 255,
}

binread_enum!(ERSDeployMode, u8);

// FINAL CLASSIFICATION
#[derive(Debug, BinRead)]
pub struct FinalClassification {
    pub header: Header,
    pub number_of_cars: u8, // Number of cars in the final classification
    #[br(count = 22)]
    pub final_classification_data: Vec<FinalClassificationData>,
}

player_data!(
    FinalClassification,
    FinalClassificationData,
    final_classification_data
);

#[derive(Debug, Default, BinRead)]
pub struct FinalClassificationData {
    pub position: u8,                // Finishing position
    pub number_of_laps: u8,          // Number of laps completed
    pub grid_position: u8,           // Grid position of the car
    pub points: u8,                  // Number of points scored
    pub number_of_pit_stops: u8,     // Number of pit stops made
    pub result_status: ResultStatus, // Result status - 0 = invalid, 1 = inactive, 2 = active
    // 3 = finished, 4 = didnotfinish, 5 = disqualified
    // 6 = not classified, 7 = retired
    pub best_lap_time_ms: u32, // Best lap time of the session in milliseconds
    pub total_race_time: f64,  // Total race time in seconds without penalties
    pub penalties_time_s: u8,  // Total penalties accumulated in seconds
    pub number_of_penalties: u8, // Number of penalties applied to this driver
    pub number_of_tyre_stints: u8, // Number of tyres stints up to maximum
    #[br(count = 8)]
    pub tyre_stints_actual: Vec<TyreCompound>, // Actual tyres used by this driver
    #[br(count = 8)]
    pub tyre_stints_visual: Vec<TyreVisual>, // Visual tyres used by this driver
    #[br(count = 8)]
    pub tyre_stints_end_laps: Vec<u8>, // The lap number stints end on
}

// LOBBY INFO
#[derive(Debug, BinRead)]
pub struct LobbyInfo {
    pub header: Header,
    pub number_of_players: u8, // Number of players in the lobby data
    #[br(count = 22)]
    pub lobby_players: Vec<LobbyInfoData>,
}

player_data!(LobbyInfo, LobbyInfoData, lobby_players);

impl LobbyInfo {
    pub fn players(self) -> Vec<LobbyInfoData> {
        let number_of_players = self.number_of_players as usize;
        self.lobby_players
            .into_iter()
            .take(number_of_players)
            .collect()
    }
}

#[derive(Debug, Default, BinRead)]
pub struct LobbyInfoData {
    #[br(map = |x: u8| x > 0)]
    pub ai_controlled: bool, // Whether the vehicle is AI (1) or Human (0) controlled
    pub team: Team, // Team id - see appendix (255 if no team currently selected)
    pub nationality: Nationality, // Nationality of the driver
    #[br(parse_with = participant_name_parser)]
    pub name: String, // Name of participant in UTF-8 format – null terminated
    // Will be truncated with ... (U+2026) if too long
    pub car_number: u8,      // Car number of the player
    pub status: LobbyStatus, // 0 = not ready, 1 = ready, 2 = spectating
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum LobbyStatus {
    NotReady,
    Ready,
    Spectating,
    #[default]
    Unknown,
}

binread_enum!(LobbyStatus, u8);

// CAR DAMAGE
#[derive(Debug, BinRead)]
pub struct CarDamage {
    pub header: Header,
    #[br(count = 22)]
    pub car_damage_data: Vec<CarDamageData>,
}

player_data!(CarDamage, CarDamageData, car_damage_data);

#[derive(Debug, Default, BinRead)]
pub struct CarDamageData {
    pub tyres_wear: WheelValue<u8>,    // Tyre wear (percentage)
    pub tyres_damage: WheelValue<u8>,  // Tyre damage (percentage)
    pub brakes_damage: WheelValue<u8>, // Brakes damage (percentage)
    pub wing_damage: WingValue<u8>,    // Wing damage (percentage)
    pub floor_damage: u8,              // Floor damage (percentage)
    pub diffuser_damage: u8,           // Diffuser damage (percentage)
    pub sidepod_damage: u8,            // Sidepod damage (percentage)
    #[br(map = |x: u8| x > 0)]
    pub drs_fault: bool, // Indicator for DRS fault, 0 = OK, 1 = fault
    #[br(map = |x: u8| x > 0)]
    pub ers_fault: bool, // Indicator for ERS fault, 0 = OK, 1 = fault
    pub gear_box_damage: u8,           // Gear box damage (percentage)
    pub engine_damage: u8,             // Engine damage (percentage)
    pub engine_mguh_wear: u8,          // Engine wear MGU-H (percentage)
    pub engine_es_wear: u8,            // Engine wear ES (percentage)
    pub engine_ce_wear: u8,            // Engine wear CE (percentage)
    pub engine_ice_wear: u8,           // Engine wear ICE (percentage)
    pub engine_mguk_wear: u8,          // Engine wear MGU-K (percentage)
    pub engine_tc_wear: u8,            // Engine wear TC (percentage)
    #[br(map = |x: u8| x > 0)]
    pub engine_blown: bool, // Engine blown, 0 = OK, 1 = fault
    #[br(map = |x: u8| x > 0)]
    pub engine_seized: bool, // Engine seized, 0 = OK, 1 = fault
}

// SESSION HISTORY
#[derive(Debug, BinRead)]
pub struct SessionHistory {
    pub header: Header,            // Header
    pub car_index: u8,             // Index of the car this lap data relates to
    pub num_laps: u8,              // Num laps in the data (including current partial lap)
    pub num_tyre_stints: u8,       // Number of tyre stints in the data
    pub best_lap_time_lap_num: u8, // Lap the best lap time was achieved on
    pub best_sector1_lap_num: u8,  // Lap the best Sector 1 time was achieved on
    pub best_sector2_lap_num: u8,  // Lap the best Sector 2 time was achieved on
    pub best_sector3_lap_num: u8,  // Lap the best Sector 3 time was achieved on
    #[br(count = 100)]
    pub lap_history_data: Vec<LapHistoryData>, // 100 laps of data max
    #[br(count = 8)]
    pub tyre_stints_history_data: Vec<TyreStintHistoryData>,
}

#[derive(Debug, Default, BinRead)]
pub struct LapHistoryData {
    pub lap_time_ms: u32,                 // Lap time in milliseconds
    pub sector_times_ms: (u16, u16, u16), // Sector times in milliseconds

    #[br(parse_with = lap_valid_flags_aprser)]
    pub lap_valid_bit_flags: LapValidFlags, // 0x01 bit set-lap valid,      0x02 bit set-sector 1 valid
                                            // 0x04 bit set-sector 2 valid, 0x08 bit set-sector 3 valid
}

bitflags! {
    #[derive(Debug)]
    pub struct LapValidFlags: u8 {
        const LAP_VALID         = 0x01;
        const SECTOR_1_VALID    = 0x02;
        const SECTOR_2_VALID    = 0x04;
        const SECTOR_3_VALID    = 0x08;
    }
}

impl Default for LapValidFlags {
    fn default() -> Self {
        LapValidFlags::empty()
    }
}

fn lap_valid_flags_aprser<R: binread::io::Read + binread::io::Seek>(
    reader: &mut R,
    _: &binread::ReadOptions,
    _: (),
) -> binread::BinResult<LapValidFlags> {
    let mut bytes: [u8; 1] = [0; 1];
    reader.read_exact(&mut bytes)?;

    Ok(LapValidFlags::from_bits(bytes[0]).unwrap_or_default())
}

#[derive(Debug, Default, BinRead)]
pub struct TyreStintHistoryData {
    pub end_lap: u8, // Lap the tyre usage ends on (255 of current tyre)
    pub tyre_actual_compound: TyreCompound, // Actual tyres used by this driver
    pub tyre_visual_compound: TyreVisual, // Visual tyres used by this driver
}

// PARSING
impl TelemetryEvent for F1_2022 {
    fn from_packet(packet: &TelemetryPacket) -> Result<F1_2022, Box<dyn Error>> {
        if packet.len() < 24 {
            return Err(Box::from("Packet is too small to contain a header"));
        }

        let packet_id = packet[5]; // packet_id
        let mut reader = Cursor::new(packet);
        match packet_id {
            0 => {
                let data: Motion = reader.read_le()?;
                Ok(F1_2022::Motion(data))
            }
            1 => {
                let data: Session = reader.read_le()?;
                Ok(F1_2022::Session(data))
            }
            2 => {
                let data: LapData = reader.read_le()?;
                Ok(F1_2022::LapData(data))
            }
            3 => {
                let data: Event = reader.read_le()?;
                Ok(F1_2022::Event(data))
            }
            4 => {
                let data: Participants = reader.read_le()?;
                Ok(F1_2022::Participants(data))
            }
            5 => {
                let data: CarSetup = reader.read_le()?;
                Ok(F1_2022::CarSetup(data))
            }
            6 => {
                let data: CarTelemetry = reader.read_le()?;
                Ok(F1_2022::CarTelemetry(data))
            }
            7 => {
                let data: CarStatus = reader.read_le()?;
                Ok(F1_2022::CarStatus(data))
            }
            8 => {
                let data: FinalClassification = reader.read_le()?;
                Ok(F1_2022::FinalClassification(data))
            }
            9 => {
                let data: LobbyInfo = reader.read_le()?;
                Ok(F1_2022::LobbyInfo(data))
            }
            10 => {
                let data: CarDamage = reader.read_le()?;
                Ok(F1_2022::CarDamage(data))
            }
            11 => {
                let data: SessionHistory = reader.read_le()?;
                Ok(F1_2022::SessionHistory(data))
            }
            id => Err(Box::from(format!("Unknown packet type: {}", id))),
        }
    }
}
