use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ReportCO2Data {
    pub measurement: u32 // ppm
}

#[derive(Serialize, Deserialize)]
pub struct RequestCO2Data;

#[derive(Serialize, Deserialize)]
pub struct CO2DataResponse {
    pub measurement: u32 // ppm
}