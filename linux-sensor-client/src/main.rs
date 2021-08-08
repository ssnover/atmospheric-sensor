use std::collections::VecDeque;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use tokio::io::AsyncReadExt;

use serial_protocol::{Header, MessageType, ReportCO2Data};

#[tokio::main]
async fn main() {
    let builder = serialport::new("/dev/ttyACM0", 115200);
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
    let mut leftover_buffer = VecDeque::new();
    loop {
        let (mut msg_buffer, packet_length) =
            listen_for_msg(&mut port, &mut leftover_buffer).await?;
        msg_buffer.resize(packet_length, 0);
        println!("Received packet with {} bytes", msg_buffer.len());
        if let Ok((header, msg)) = postcard::take_from_bytes::<Header>(&msg_buffer[..]) {
            match header.msg_type {
                MessageType::ReportCO2Data => {
                    let (co2_data, _remaining) =
                        postcard::take_from_bytes::<ReportCO2Data>(&msg).unwrap();
                    println!("CO2 measurement: {} ppm", co2_data.measurement);
                }
                _ => {
                    eprintln!("Unexpected message type received");
                }
            }
        }
    }
}

async fn listen_for_msg(
    port: &mut tokio::fs::File,
    leftover_buffer: &mut VecDeque<u8>,
) -> std::io::Result<(Vec<u8>, usize)> {
    let mut input_buffer = [0u8; 1024];
    let mut output_buffer = vec![0u8; 1024];
    let mut decoder = Box::new(cobs::CobsDecoder::new(&mut output_buffer));

    while !leftover_buffer.is_empty() {
        match decoder.feed(leftover_buffer.pop_front().unwrap()) {
            Ok(None) => {
                // message still in progress
                continue;
            }
            Ok(Some(bytes_decoded)) => {
                return Ok((output_buffer, bytes_decoded));
            }
            Err(_) => {
                continue;
            }
        }
    }

    loop {
        let bytes_read = port.read(&mut input_buffer).await?;
        let mut bytes_in_packet = None;
        for index in 0..bytes_read {
            if bytes_in_packet.is_none() {
                match decoder.feed(input_buffer[index]) {
                    Ok(None) => {
                        // message still in progress
                        continue;
                    }
                    Ok(Some(bytes_decoded)) => {
                        bytes_in_packet = Some(bytes_decoded);
                    }
                    Err(_) => {
                        continue;
                    }
                }
            } else {
                // put the remaining bytes into the leftover buffer
                leftover_buffer.push_back(input_buffer[index]);
            }
        }

        if let Some(bytes_in_packet) = bytes_in_packet {
            return Ok((output_buffer, bytes_in_packet));
        }
    }
}
