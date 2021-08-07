use std::path::Path;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use linux_sensor_sim::PseudoTerminal;
use serial_protocol::ReportCO2Data;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Create a fake serial port for application code to communicate to the simulator with
    let pty = PseudoTerminal::new()?;
    pty.create_symlink(Path::new("/tmp/atmospheric-sensor-sim"))?;

    tokio::select! {
        _ = simulator_run(pty) => {
            println!("Simulator exited");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received SIGINT");
        }
    }

    Ok(())
}

async fn simulator_run(pty: PseudoTerminal) {
    let (mut server_serial_tx, server_serial_rx) = async_channel::unbounded();
    let (client_serial_tx, _client_serial_rx) = async_channel::unbounded();

    tokio::spawn(async move {
        serial_comm_server(pty, server_serial_rx, client_serial_tx).await;
    });

    loop {
        // Periodically report simulated data, exit if there's an error writing the data
        match tokio::join!(
            report_co2_data(&mut server_serial_tx),
            sleep(Duration::from_millis(100))
        ) {
            (Ok(()), _) => {
                continue;
            }
            (Err(e), _) => {
                eprintln!("Error while reporting data: {}", e);
                break;
            }
        }
    }
}

async fn report_co2_data(tx: &mut Sender<Vec<u8>>) -> std::io::Result<()> {
    let measurement: f32 = 300.;
    let msg = ReportCO2Data { measurement };
    let msg = serial_protocol::Message { header: serial_protocol::Header { version: 0x00, id: 0x00, msg_type: serial_protocol::MessageType::ReportCO2Data }, message: msg };
    let write_buffer = postcard::to_stdvec(&msg).unwrap();
    tx.send(write_buffer).await.unwrap();
    Ok(())
}

async fn serial_comm_server(pty: PseudoTerminal, rx: Receiver<Vec<u8>>, tx: Sender<Vec<u8>>) {
    let pty = Arc::new(Mutex::new(pty));
    tokio::select! {
        _ = serve_serial_port(Arc::clone(&pty), tx) => {
            eprintln!("Error while serving serial port");
        },
        _ = recv_client_task(Arc::clone(&pty), rx) => {
            eprintln!("Error while running serial port client task");
        }
    }
}

async fn serve_serial_port(mut pty: Arc<Mutex<PseudoTerminal>>, tx: Sender<Vec<u8>>) {
    loop {
        tokio::select! {
            _ = sleep(Duration::from_millis(1)) => {}
            v = server_context(&mut pty) => { 
                    if v.len() > 0 {
                        tx.send(v).await.unwrap(); 
                    }
                }
        }
    }
}

async fn server_context(pty: &mut Arc<Mutex<PseudoTerminal>>) -> Vec<u8> {
    let mut pty = pty.lock().await;
    let mut contents = vec![];
    let _bytes_read = pty.master_file.read(&mut contents[..]).await.unwrap();
    contents
}

async fn recv_client_task(pty: Arc<Mutex<PseudoTerminal>>, rx: Receiver<Vec<u8>>) {
    loop {
        match rx.recv().await {
            Ok(msg) => {
                let msg = cobs::encode_vec(&msg[..]);
                let mut pty = pty.lock().await;
                pty.master_file.write_all(&msg[..]).await.unwrap();
            }
            Err(_) => {
                break;
            }
        }
    }
}
