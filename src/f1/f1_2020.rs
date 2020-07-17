use std::error::Error;

use crate::{TelemetryEvent, TelemetryPacket};

// extern crate byteorder;
// use byteorder::{LittleEndian};

/// F1_2020 implements the codemasters UDP telemetry protocol for "F1 2020"
/// see: https://forums.codemasters.com/topic/50942-f1-2020-udp-specification/
pub enum F1_2020 {
    Motion(Header),
    Session(Header),
    LapData(Header),
    Event(Header),
    Participants(Header),
    CarSetups(Header),
    CarTelemetry(Header),
    CarStatus(Header),
    FinalClassification(Header),
    LobbyInfo(Header),
}

pub struct Header {
    packet_format: u16,
    game_major_version: u8,
    game_minor_version: u8,
    packet_version: u8,
    packet_id: u8,
    session_uid: u64,
    session_time: f32,
    frame_identifier: u32,
    player_car_index: u8,
    secondary_player_car_index: u8,
}

impl TelemetryEvent for F1_2020 {
    fn from_packet(_packet: &TelemetryPacket) -> Result<F1_2020, Box<dyn Error>> {
        let header = Header {};
        match header.packet_id {
            0 => Ok(F1_2020::Motion(header)),
            1 => Ok(F1_2020::Session(header)),
            2 => Ok(F1_2020::LapData(header)),
            3 => Ok(F1_2020::Event(header)),
            4 => Ok(F1_2020::Participants(header)),
            5 => Ok(F1_2020::CarSetups(header)),
            6 => Ok(F1_2020::CarTelemetry(header)),
            7 => Ok(F1_2020::CarStatus(header)),
            8 => Ok(F1_2020::FinalClassification(header)),
            9 => OK(F1_2020::LobbyInfo(header)),
            id => Err(format!("Unknown packet type: {}", id)),
        }
    }
}
