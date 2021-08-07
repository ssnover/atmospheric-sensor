#![no_std]

use serde::{Deserialize, Serialize};

#[repr(u16)]
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    // 0x00xx reserved
    // 0x01xx - Environment Data
    ReportCO2Data = 0x0100,
    RequestCO2Data = 0x0101,
    CO2DataResponse = 0x0102,
}

#[derive(Serialize, Deserialize)]
pub struct Header {
    pub version: u8,
    pub id: u8,
    pub msg_type: MessageType,
}

#[derive(Serialize, Deserialize)]
pub struct Message<T> {
    pub header: Header,
    pub message: T,
}

#[derive(Serialize, Deserialize)]
pub struct ReportCO2Data {
    pub measurement: f32, // ppm
}

impl ReportCO2Data {
    pub fn new(data: f32) -> Self {
        ReportCO2Data { measurement: data }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RequestCO2Data;

#[derive(Serialize, Deserialize)]
pub struct CO2DataResponse {
    pub measurement: f32, // ppm
}
