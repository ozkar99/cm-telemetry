use std::error::Error;

use crate::{TelemetryEvent, TelemetryPacket};

extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

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

pub struct Motion {
    pub header: Header,
}

pub struct Session {
    pub header: Header,
}

pub struct LapData {
    pub header: Header,
}

pub struct Event {
    pub header: Header,
}

pub struct Participants {
    pub header: Header,
}

pub struct CarSetups {
    pub header: Header,
}

pub struct CarTelemetry {
    pub header: Header,
}

pub struct CarStatus {
    pub header: Header,
}

pub struct FinalClassification {
    pub header: Header,
}

pub struct LobbyInfo {
    pub header: Header,
}

impl TelemetryEvent for F1_2020 {
    fn from_packet(packet: &TelemetryPacket) -> Result<F1_2020, Box<dyn Error>> {
        let header = Header::from_packet(packet)?;
        match header.packet_id {
            0 => Ok(F1_2020::Motion(Motion { header })),
            1 => Ok(F1_2020::Session(Session { header })),
            2 => Ok(F1_2020::LapData(LapData { header })),
            3 => Ok(F1_2020::Event(Event { header })),
            4 => Ok(F1_2020::Participants(Participants { header })),
            5 => Ok(F1_2020::CarSetups(CarSetups { header })),
            6 => Ok(F1_2020::CarTelemetry(CarTelemetry { header })),
            7 => Ok(F1_2020::CarStatus(CarStatus { header })),
            8 => Ok(F1_2020::FinalClassification(FinalClassification { header })),
            9 => Ok(F1_2020::LobbyInfo(LobbyInfo { header })),
            id => Err(Box::from(format!("Unknown packet type: {}", id))),
        }
    }
}

impl Header {
    fn from_packet(packet: &TelemetryPacket) -> Result<Header, Box<dyn Error>> {
        if packet.len() < 24 {
            return Err(Box::from("Packet is too small to contain a header"));
        }
        Ok(Header {
            packet_format: LittleEndian::read_u16(&packet[0..2]),
            game_major_version: packet[2],
            game_minor_version: packet[3],
            packet_version: packet[4],
            packet_id: packet[5],
            session_uid: LittleEndian::read_u64(&packet[6..14]),
            session_time: LittleEndian::read_f32(&packet[14..18]),
            frame_identifier: LittleEndian::read_u32(&packet[18..22]),
            player_car_index: packet[22],
            secondary_player_car_index: packet[23],
        })
    }
}
