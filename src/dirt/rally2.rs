use crate::{Event, Packet};
use std::error::Error;

extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub struct DirtRally2 {
    pub car: Car,
    pub session: Session,
    pub motion: Motion,
}

impl Event for DirtRally2 {
    // see: https://docs.google.com/spreadsheets/d/1eA518KHFowYw7tSMa-NxIFYpiWe5JXgVVQ_IMs7BVW0/edit#gid=0 for details on the specification
    fn from_packet(packet: &Packet) -> Result<DirtRally2, Box<dyn Error>> {
        if packet.len() < 256 {
            return Err(Box::from("Packet size is less than 256 bytes, please set extradata=3 on hardware_settings_config.xml"));
        }
        Ok(DirtRally2 {
            car: Car::from_packet(&packet)?,
            session: Session::from_packet(&packet)?,
            motion: Motion::from_packet(&packet)?,
        })
    }
}

#[derive(Debug)]
pub struct Session {
    pub position: f32,
    pub location: Coordinate,
    pub track: Track,
    pub lap_info: Lap,
}

#[derive(Debug)]
pub struct Car {
    pub speed: f32,
    pub gear: Gear,
    pub wheels: (Wheel, Wheel, Wheel, Wheel), // Rear-Left, Rear-Right, Front-Left, Front-Right
    pub throttle: f32,
    pub steer: f32,
    pub brake: f32,
    pub clutch: f32,
    pub rpms: f32,
}

#[derive(Debug)]
pub enum Gear {
    Reverse,
    Neutral,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eigth,
    Ninth,
}

#[derive(Debug)]
pub struct Motion {
    pub velocity: Coordinate,
    pub roll_vector: Coordinate,
    pub pitch_vector: Coordinate,
    pub g_force_lateral: f32,
    pub g_force_longitudinal: f32,
}

#[derive(Debug)]
pub struct Wheel {
    pub suspension_position: f32,
    pub suspension_velocity: f32,
    pub wheel_velocity: f32,
    pub brake_temperature: f32,
}

#[derive(Debug)]
pub struct Track {
    pub time: f32,
    pub distance: f32,
    pub length: f32,
}

#[derive(Debug)]
pub struct Lap {
    pub current_lap: f32,
    pub total_laps: f32,
    pub last_lap_time: f32,
    pub current_lap_time: f32,
    pub current_lap_distance: f32,
}

type Coordinate = (f32, f32, f32); // x,y,z coordinates

impl Car {
    fn from_packet(packet: &Packet) -> Result<Car, Box<dyn Error>> {
        Ok(Car {
            speed: LittleEndian::read_f32(&packet[28..32]),
            throttle: LittleEndian::read_f32(&packet[116..120]),
            steer: LittleEndian::read_f32(&packet[120..124]),
            brake: LittleEndian::read_f32(&packet[124..128]),
            clutch: LittleEndian::read_f32(&packet[128..132]),
            rpms: LittleEndian::read_f32(&packet[148..152]),
            gear: Gear::from_f32(LittleEndian::read_f32(&packet[132..136]))?,
            wheels: (
                Wheel {
                    // Rear-Left
                    suspension_position: LittleEndian::read_f32(&packet[68..72]),
                    suspension_velocity: LittleEndian::read_f32(&packet[84..88]),
                    wheel_velocity: LittleEndian::read_f32(&packet[100..104]),
                    brake_temperature: LittleEndian::read_f32(&packet[204..208]),
                },
                Wheel {
                    // Rear-Right
                    suspension_position: LittleEndian::read_f32(&packet[72..76]),
                    suspension_velocity: LittleEndian::read_f32(&packet[88..92]),
                    wheel_velocity: LittleEndian::read_f32(&packet[104..108]),
                    brake_temperature: LittleEndian::read_f32(&packet[208..212]),
                },
                Wheel {
                    // Front-Left
                    suspension_position: LittleEndian::read_f32(&packet[76..80]),
                    suspension_velocity: LittleEndian::read_f32(&packet[92..96]),
                    wheel_velocity: LittleEndian::read_f32(&packet[108..112]),
                    brake_temperature: LittleEndian::read_f32(&packet[212..216]),
                },
                Wheel {
                    // Front-Right
                    suspension_position: LittleEndian::read_f32(&packet[80..84]),
                    suspension_velocity: LittleEndian::read_f32(&packet[96..100]),
                    wheel_velocity: LittleEndian::read_f32(&packet[112..116]),
                    brake_temperature: LittleEndian::read_f32(&packet[216..220]),
                },
            ),
        })
    }
}

impl Session {
    fn from_packet(packet: &Packet) -> Result<Session, Box<dyn Error>> {
        Ok(Session {
            location: (
                LittleEndian::read_f32(&packet[16..20]),
                LittleEndian::read_f32(&packet[20..24]),
                LittleEndian::read_f32(&packet[24..28]),
            ),
            position: LittleEndian::read_f32(&packet[156..160]),
            track: Track::from_packet(&packet)?,
            lap_info: Lap::from_packet(&packet)?,
        })
    }
}

impl Motion {
    fn from_packet(packet: &Packet) -> Result<Motion, Box<dyn Error>> {
        Ok(Motion {
            g_force_lateral: LittleEndian::read_f32(&packet[136..140]),
            g_force_longitudinal: LittleEndian::read_f32(&packet[140..144]),
            pitch_vector: (
                LittleEndian::read_f32(&packet[56..60]),
                LittleEndian::read_f32(&packet[60..64]),
                LittleEndian::read_f32(&packet[64..68]),
            ),
            roll_vector: (
                LittleEndian::read_f32(&packet[44..48]),
                LittleEndian::read_f32(&packet[48..52]),
                LittleEndian::read_f32(&packet[52..56]),
            ),
            velocity: (
                LittleEndian::read_f32(&packet[32..36]),
                LittleEndian::read_f32(&packet[36..40]),
                LittleEndian::read_f32(&packet[40..44]),
            ),
        })
    }
}

impl Lap {
    fn from_packet(packet: &Packet) -> Result<Lap, Box<dyn Error>> {
        Ok(Lap {
            current_lap_time: LittleEndian::read_f32(&packet[4..8]),
            current_lap_distance: LittleEndian::read_f32(&packet[8..12]),
            current_lap: LittleEndian::read_f32(&packet[144..148]),
            total_laps: LittleEndian::read_f32(&packet[240..244]),
            last_lap_time: LittleEndian::read_f32(&packet[248..252]),
        })
    }
}

impl Track {
    fn from_packet(packet: &Packet) -> Result<Track, Box<dyn Error>> {
        Ok(Track {
            distance: LittleEndian::read_f32(&packet[12..16]),
            time: LittleEndian::read_f32(&packet[0..4]),
            length: LittleEndian::read_f32(&packet[244..248]),
        })
    }
}

impl Gear {
    fn from_f32(f: f32) -> Result<Gear, Box<dyn Error>> {
        if f < 0.0 {
            return Ok(Gear::Reverse);
        }

        if f >= 0.0 && f < 1.0 {
            return Ok(Gear::Neutral);
        }

        if f >= 1.0 && f < 2.0 {
            return Ok(Gear::First);
        }

        if f >= 2.0 && f < 3.0 {
            return Ok(Gear::Second);
        }

        if f >= 3.0 && f < 4.0 {
            return Ok(Gear::Third);
        }

        if f >= 4.0 && f < 5.0 {
            return Ok(Gear::Fourth);
        }

        if f >= 5.0 && f < 6.0 {
            return Ok(Gear::Fifth);
        }

        if f >= 6.0 && f < 7.0 {
            return Ok(Gear::Sixth);
        }

        if f >= 7.0 && f < 8.0 {
            return Ok(Gear::Seventh);
        }

        if f >= 8.0 && f < 9.0 {
            return Ok(Gear::Eigth);
        }

        if f >= 9.0 {
            return Ok(Gear::Ninth);
        }

        Err(Box::from("unknown gear"))
    }
}
