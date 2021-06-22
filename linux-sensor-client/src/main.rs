use std::os::unix::io::{FromRawFd, IntoRawFd};
use tokio::io::{AsyncReadExt};

use serial_protocol::{Header, MessageType, ReportCO2Data};

#[tokio::main]
async fn main() {
    let builder = serialport::new("/tmp/atmospheric-sensor-sim", 115200);
    let port = builder.open_native().unwrap();
    let mut port = unsafe {tokio::fs::File::from_raw_fd(port.into_raw_fd()) };

    tokio::select! {
        _ = listen(&mut port) => {
            println!("Simulator exited");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received SIGINT");
        }
    }
}

async fn listen(port: &mut tokio::fs::File) -> std::io::Result<()> {
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        let num_bytes = port.read(&mut buf[..]).await?;
        println!("Received {} bytes", num_bytes);
        let msg = cobs::decode_vec(&buf[..num_bytes]).unwrap();
        let (header, msg) = postcard::take_from_bytes::<Header>(&msg[..]).unwrap();
        if header.msg_type == MessageType::ReportCO2Data {
            let (data, _remaining) = postcard::take_from_bytes::<ReportCO2Data>(&msg[..]).unwrap();
            println!("Data: 0x{:x}", data.measurement);
        }
    }
}
