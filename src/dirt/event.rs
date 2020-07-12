use std::error::Error;

#[derive(Debug)]
pub struct Event {
  session: Session,
  car: Car,
  physics: Physics,
}

#[derive(Debug)]
pub struct Session {
  position: f32,
  location: (f32, f32, f32), // x,y,z
  track: Track,
  stage: Stage,
  sector: Sector,
  lap_info: Lap,
}

#[derive(Debug)]
pub struct Car {
  speed: f32,
  gear: Gear,
  wheels: (Wheel, Wheel, Wheel, Wheel), // Rear-Right, Rear-Left, Front-Right, Front-Left
  throttle: f32,
  steer: f32,
  brake: f32,
  clutch: f32,
  rpms: f32,
}

#[derive(Debug)]
pub enum Gear {
  Neutral,
  First,
  Second,
  Third,
  Fourth,
  Fifth,
  Sixth,
  Seventh,
  Eigth,
  Nine,
  Reverse,
}

#[derive(Debug)]
pub struct Physics {
  // x, y, z
  velocity: (f32, f32, f32),
  roll_vector: (f32, f32, f32),
  pitch_vector: (f32, f32, f32),
  g_force_lateral: f32,
  g_force_longitudinal: f32,
}

#[derive(Debug)]
pub struct Wheel {
  suspension_position: f32,
  suspension_velocity: f32,
  wheel_position: f32,
  wheel_velocity: f32,
  brake_temperature: f32,
}

#[derive(Debug)]
pub struct Track {
  time: f32,
  distance: f32,
  length: f32,
}

#[derive(Debug)]
pub struct Stage {
  time: f32,
  distance: f32,
}

#[derive(Debug)]
pub struct Sector {
  time: f32,
  second_time: f32,
}

#[derive(Debug)]
pub struct Lap {
  current_lap: f32,
  total_laps: f32,
  last_lap_time: f32,
}

impl Event {
  pub fn from_packet(packet: Vec<u8>) -> Result<Event, Box<dyn Error>> {
    // see: https://docs.google.com/spreadsheets/d/1eA518KHFowYw7tSMa-NxIFYpiWe5JXgVVQ_IMs7BVW0/edit#gid=0
    return Err(Box::from("generic error"))
  }
}