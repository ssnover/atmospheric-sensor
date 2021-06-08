use std::path::Path;

use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};

use linux_sensor_sim::PseudoTerminal;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Create a fake serial port for application code to communicate to the simulator with
    let mut pty = PseudoTerminal::new()?;
    pty.create_symlink(Path::new("/tmp/atmospheric-sensor-sim"))?;

    tokio::select! {
        _ = simulator_run(&mut pty) => {
            println!("Simulator exited");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received SIGINT");
        }
    }

    Ok(())
}

async fn simulator_run(pty: &mut PseudoTerminal) {
    loop {
        // Periodically report simulated data, exit if there's an error writing the data
        match tokio::join!(report_co2_data(pty), sleep(Duration::from_millis(100))) {
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

async fn report_co2_data(pty: &mut PseudoTerminal) -> std::io::Result<()> {
    let write_buffer: [u8; 8] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    pty.master_file.write_all(&write_buffer).await?;
    Ok(())
}
