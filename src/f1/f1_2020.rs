use std::convert::TryFrom;
use std::error::Error;
use std::io::Cursor;

use crate::{
    TelemetryEvent,
    TelemetryPacket,
    f1::util::*,
    f1::macros::*,
};

use binread::{BinRead, BinReaderExt};
use num_enum::TryFromPrimitive;

/// F1_2020 implements the codemasters UDP telemetry protocol for "F1 2020"
/// see: https://forums.codemasters.com/topic/50942-f1-2020-udp-specification/
pub enum F1_2020 {
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

#[derive(Debug, BinRead)]
pub struct Motion {
    pub header: Header,
    #[br(count = 22)]
    pub car_motion_data: Vec<CarMotionData>,
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

player_data!(Motion, CarMotionData, car_motion_data);

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
    pub session_type: SessionType,
    pub track: Track,
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

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Weather {
    Clear,
    LigthCloud,
    Overcast,
    LightRain,
    HeavyRain,
    Storm,
    #[default]
    Unknown = 255,
}

binread_enum!(Weather, u8);

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
}

binread_enum!(Track, i8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Formula {
    F1Modern,
    F1Classic,
    F2,
    F1Generic,
    #[default]
    Unknown = 255,
}

binread_enum!(Formula, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum SafetyCarStatus {
    NoSafetyCar,
    FullSafetyCar,
    VirtualSafetyCar,
    #[default]
    Unknown = 255,
}

binread_enum!(SafetyCarStatus, u8);

#[derive(Debug, Default, BinRead)]
pub struct MarshalZone {
    pub zone_start: f32,
    pub zone_flag: ZoneFlag,
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

#[derive(Debug, Default, BinRead)]
pub struct WeatherForecastSample {
    pub session_type: SessionType,
    pub time_offset: u8,
    pub weather: Weather,
    pub track_temperature: i8,
    pub air_temperature: i8,
}

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
    TimeTrial,
}

binread_enum!(SessionType, u8);

#[derive(Debug, BinRead)]
pub struct LapData {
    pub header: Header,
    #[br(count = 22)]
    pub laps: Vec<Lap>,
}

player_data!(LapData, Lap, laps);

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
    pub pit_status: PitStatus,
    pub sector: Sector,
    #[br(map = |x: u8| x > 0)]
    pub current_lap_invalid: bool,
    pub penalties: u8,
    pub grid_position: u8,
    pub driver_status: DriverStatus,
    pub result_status: ResultStatus,
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

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum PitStatus {
    None,
    Pitting,
    InPitArea,
    #[default]
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
    InTrack,
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
    Disqualified,
    NotClassified,
    Retired,
    MechanicalFailure,
    #[default]
    Unknown = 255,
}

binread_enum!(ResultStatus, u8);

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
    pub penalty_type: PenaltyType,
    pub infrigement_type: InfringementType,
    pub vehicle_index: u8,
    pub other_vehicle_index: u8,
    pub time: u8,
    pub lap_number: u8,
    pub places_gained: u8,
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
    #[default]
    Unknown = 255,
}

binread_enum!(InfringementType, u8);

#[derive(Debug, BinRead)]
pub struct Participants {
    pub header: Header,
    pub num_active_cars: u8,
    #[br(count = 22)]
    pub participants_data: Vec<ParticipantsData>,
}

player_data!(Participants, ParticipantsData, participants_data);

#[derive(Debug, Default, BinRead)]
pub struct ParticipantsData {
    #[br(map = |x: u8| x > 0)]
    pub ai_controlled: bool,
    pub driver: Driver,
    pub team: Team,
    pub race_number: u8,
    pub nationality: Nationality,
    #[br(parse_with = participant_name_parser)]
    pub name: String,
    #[br(map = |x: u8| x > 1)]
    pub your_telemetry_restricted: bool,
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
    KimiRaikkonen = 6, // Kimi Räikkönen
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
    NaotaIzumi,
    HowardClarke,
    WilheimKaufmann,
    MarieLaursen,
    FlavioNieves,
    PeterBelousovm,
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
    SergioSetteCamara = 55, // Sérgio Sette Câmara
    LouisDeletraz,          // Louis Delétraz
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
    AntonioGiovinazzi = 74,
    RobertKubica,
    NobuharuMatsushita = 78,
    NikitaMazepin,
    GuanyaZhou,
    MickSchumacher,
    CallumIlott,
    JuanManuel,
    Correa,
    JordanKing,
    MahaveerRaghunathan,
    TatianaCalderon,
    AnthoineHubert,
    GuilianoAlesi,
    RalphBoschung,
    MyDriver = 100,
    #[default]
    Unknown = 255, // Used for time trial "ghost" drivers that appear randomly
}

binread_enum!(Driver, u8);

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum Team {
    Mercedes,
    Ferrari,
    RedBullRacing,
    Williams,
    RacingPoint,
    Renault,
    AlphaTauri,
    Haas,
    McLaren,
    AlfaRomeo,
    McLaren1988,
    McLaren1991,
    Williams1992,
    Ferrari1995,
    Williams1996,
    McLaren1998,
    Ferrari2002,
    Ferrari2004,
    Renault2006,
    Ferrari2007,
    McLaren2008,
    RedBull2010,
    Ferrari1976,
    ARTGrandPrix,
    CamposVexatecRacing,
    Carlin,
    CharouzRacingSystem,
    DAMS,
    RussianTime,
    MPMotorsport,
    Pertamina,
    McLaren1990,
    Trident,
    BWTArden,
    McLaren1976,
    Lotus1972,
    Ferrari1979,
    McLaren1982,
    Williams2003,
    Brawn2009,
    Lotus1978,
    F1GenericCar,
    ArtGP19,
    Campos19,
    Carlin19,
    SauberJuniorCharouz19,
    Dams19,
    UniVirtuosi19,
    MPMotorsport19,
    Prema19,
    Trident19,
    Arden19,
    Benetton1994,
    Benetton1995,
    Ferrari2000,
    Jordan1991,
    Ferrari1990 = 63,
    McLaren2010,
    Ferrari2010,
    #[default]
    Unknown = 254,
    MyTeam = 255,
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
    NorthKorean,
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
    Welsh,
    Barbadian,
    Vietnamese,
}

binread_enum!(Nationality, u8);

#[derive(Debug, BinRead)]
pub struct CarSetup {
    pub header: Header,
    #[br(count = 22)]
    pub car_setup_data: Vec<CarSetupData>,
}

#[derive(Debug, Default, BinRead)]
pub struct CarSetupData {
    pub wing: FrontRearValue<u8>,
    pub on_throttle: u8,
    pub off_throttle: u8,
    pub camber: FrontRearValue<f32>,
    pub toe: FrontRearValue<f32>,
    pub suspension: FrontRearValue<u8>,
    pub anti_roll_bar: FrontRearValue<u8>,
    pub suspension_height: FrontRearValue<u8>,
    pub brake_pressure: u8,
    pub brake_bias: u8,
    pub type_pressure: WheelValue<f32>,
    pub ballast: u8,
    pub fuel_load: f32,
}

player_data!(CarSetup, CarSetupData, car_setup_data);

#[derive(Debug, BinRead)]
pub struct CarTelemetry {
    pub header: Header,
    #[br(count = 22)]
    pub car_telemetry_data: Vec<CarTelemetryData>,
    pub button_status: u32,
    pub mfd_panel: MFDPanel,
    pub mfd_panel_secondary_player: MFDPanel,
    #[br(map = |x: i8| if x == 0 { Gear::Unknown } else { Gear::try_from(x).unwrap() })]
    pub suggested_gear: Gear,
}

player_data!(CarTelemetry, CarTelemetryData, car_telemetry_data);

#[derive(Debug, Default, BinRead)]
pub struct CarTelemetryData {
    pub speed: u16,
    pub throttle: f32,
    pub steer: f32,
    pub brake: f32,
    pub clutch: u8,
    #[br(map = |x: i8| Gear::try_from(x).unwrap())]
    pub gear: Gear,
    pub engine_rpm: u16,
    #[br(map = |x: u8| x > 0)]
    pub drs: bool,
    pub rev_lights_percent: u8,
    pub brake_temp: WheelValue<u16>,
    pub tyres_surface_temp: WheelValue<u8>,
    pub tyres_inner_temp: WheelValue<u8>,
    pub engine_temp: u16,
    pub tyres_pressure: WheelValue<f32>,
    #[br(parse_with = surface_type_parser)]
    pub surface_type: WheelValue<Surface>,
}

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

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum MFDPanel {
    CarSetup,
    Pits,
    Damage,
    Engine,
    Temperatures,
    #[default]
    Unknown,
    Closed = 255,
}

binread_enum!(MFDPanel, u8);

#[derive(Debug, BinRead)]
pub struct CarStatus {
    pub header: Header,
    #[br(count = 22)]
    pub car_status_data: Vec<CarStatusData>,
}

player_data!(CarStatus, CarStatusData, car_status_data);

#[derive(Debug, Default, BinRead)]
pub struct CarStatusData {
    pub traction_control: u8,
    #[br(map = |x: u8| x > 0)]
    pub anti_lock_brakes: bool,
    pub fuel_mix: FuelMix,
    pub front_brake_bias: u8,
    #[br(map = |x: u8| x > 0)]
    pub pit_limiter_status: bool,
    pub fuel_in_tank: f32,
    pub fuel_capacity: f32,
    pub fuel_remaining_laps: f32,
    pub max_rpm: u16,
    pub idle_rpm: u16,
    pub max_gears: u8,
    pub drs_allowed: DRSAllowed,
    #[br(map = |x: u16| if x > 0 { DRSActivationDistance::Distance(x) } else { DRSActivationDistance::NotAvailable })]
    pub drs_activation_distance: DRSActivationDistance,
    pub tyres_wear: WheelValue<u8>,
    pub tyres_compound: TyreCompound,
    pub tyres_visual: TyreVisual,
    pub tyres_ages_lap: u8,
    pub tyres_damage: WheelValue<u8>,
    pub wing_damage: WingValue<u8>,
    #[br(map = |x: u8| x > 0)]
    pub drs_fault: bool,
    pub engine_damage: u8,
    pub gearbox_damage: u8,
    pub vehicle_fia_flag: FiaFlag,
    pub ers_data: ERS,
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

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum DRSAllowed {
    NotAllowed,
    Allowed,
    #[default]
    Unknown,
}

binread_enum!(DRSAllowed, u8);

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
    Wet = 8,
    Soft = 16,
    Medium = 17,
    Hard = 18,
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
    pub stored_energy: f32,
    pub deploy_mode: ERSDeployMode,
    pub harvested_this_lap_mguk: f32,
    pub harvested_this_lap_mguh: f32,
    pub deployed_this_lap: f32,
}

#[derive(Debug, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum ERSDeployMode {
    None,
    Medium,
    Overtake,
    Hotlap,
    #[default]
    Unknown = 255,
}

binread_enum!(ERSDeployMode, u8);

#[derive(Debug, BinRead)]
pub struct FinalClassification {
    pub header: Header,
    pub number_of_cars: u8,
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
    pub position: u8,
    pub number_of_laps: u8,
    pub grid_position: u8,
    pub points: u8,
    pub number_of_pit_stops: u8,
    pub result_status: ResultStatus,
    pub best_lap_time: f32,
    pub total_race_time: f64,
    pub penalties_time: u8,
    pub number_of_penalties: u8,
    pub number_of_tyre_stints: u8,
    #[br(count = 8)]
    pub tyre_stints_actual: Vec<TyreCompound>,
    #[br(count = 8)]
    pub tyre_stints_visual: Vec<TyreVisual>,
}

#[derive(Debug, BinRead)]
pub struct LobbyInfo {
    pub header: Header,
    pub number_of_players: u8,
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
    pub ai_controlled: bool,
    pub team: Team,
    pub nationality: Nationality,
    #[br(parse_with = participant_name_parser)]
    pub name: String,
    pub status: LobbyStatus,
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
                let data: CarSetup = reader.read_le()?;
                Ok(F1_2020::CarSetup(data))
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
