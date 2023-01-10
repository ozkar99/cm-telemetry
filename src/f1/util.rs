use binread::BinRead;

use num::Num;

#[derive(Debug, Default, BinRead)]
pub struct Coordinates<T: Num + binread::BinRead<Args = ()>> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Debug, Default, BinRead)]
pub struct WheelValue<T: binread::BinRead<Args = ()>> {
    pub rear_left: T,
    pub rear_right: T,
    pub front_left: T,
    pub front_right: T,
}

#[derive(Debug, Default, BinRead)]
pub struct FrontRearValue<T: Num + binread::BinRead<Args = ()>> {
    pub front: T,
    pub rear: T,
}

#[derive(Debug, Default, BinRead)]
pub struct WingValue<T: binread::BinRead<Args = ()>> {
    pub front_left: T,
    pub front_right: T,
    pub rear: T,
}