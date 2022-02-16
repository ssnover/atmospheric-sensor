use std::os::unix::io::{FromRawFd, IntoRawFd};
use tokio::io::AsyncReadExt;

use serial_protocol::{Header, MessageType, ReportCO2Data};

#[tokio::main]
async fn main() {
    let builder = serialport::new("/tmp/atmospheric-sensor-sim", 115200);
    let port = builder.open_native().unwrap();
    let port = unsafe { tokio::fs::File::from_raw_fd(port.into_raw_fd()) };

    tokio::select! {
        _ = listen(port) => {
            println!("Simulator exited");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received SIGINT");
        }
    }
}

async fn listen(mut port: tokio::fs::File) -> std::io::Result<()> {
    loop {
        let (header, body) = listen_for_msg(&mut port).await?;
        match header.msg_type {
            MessageType::ReportCO2Data => {
                let co2_data = serial_protocol::from_buffer::<ReportCO2Data>(&body[..]).unwrap();
                println!("CO2 measurement: {} ppm", co2_data.measurement);
            }
            _ => {
                eprintln!("Unexpected message type received");
            }
        }
    }
}

async fn listen_for_msg(
    port: &mut tokio::fs::File,
) -> std::io::Result<(Header, Vec<u8>)> {
    let mut bytes_read = 0;
    let mut input_buffer = [0 as u8; 1024];

    loop {
        bytes_read += port.read(&mut input_buffer[bytes_read..]).await?;
        println!("Read bytes: {:?}", &input_buffer[..bytes_read]);
        if let Ok((msg, _remaining)) = serial_protocol::decode_packet(&mut input_buffer[..bytes_read]) {
            return Ok((msg.hdr, msg.msg.to_vec()))
        }
    }
}
