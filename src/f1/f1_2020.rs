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

pub struct Header {}

impl TelemetryEvent for F1_2020 {
    fn from_packet(_packet: &TelemetryPacket) -> Result<F1_2020, Box<dyn Error>> {
        let header = Header {};
        Ok(F1_2020::Motion(header))
    }
}
