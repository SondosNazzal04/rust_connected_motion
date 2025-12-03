use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};
use std::process::Stdio;
use tokio::sync::mpsc;

// This function runs in the background forever
pub async fn run_listener(tx: mpsc::Sender<Vec<f64>>) {
    println!("Starting CSI Listener on /dev/ttyUSB0...");

    // 1. SPAWN THE CLI TOOL
    // We use the Linux path because you are on WSL
    let mut child = Command::new("esp-csi-cli-rs")
        .arg("--port")
        .arg("/dev/ttyUSB0")
        // Add other args if the tool needs them (e.g. baud rate)
        .stdout(Stdio::piped()) // Capture the output
        .spawn()
        .expect("Failed to start esp-csi-cli-rs. Make sure the tool is installed and in your PATH.");

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout).lines();

    // 2. READ LOOP
    while let Ok(Some(line)) = reader.next_line().await {
        // DEBUG: Uncomment this if you see no graph, to check what the ESP is sending
        // println!("Raw line: {}", line);

        // We assume the line starts with "CSI" or contains comma-separated numbers
        // Adjust "CSI" to whatever the actual prefix is (or remove the check to try parsing everything)
        if line.starts_with("CSI") {
            if let Some(parsed_data) = parse_csi_line(&line) {
                // Send data to the UI. If the UI is closed, this will error and we break the loop.
                if let Err(_) = tx.send(parsed_data).await {
                    break;
                }
            }
        }
    }
}

// 3. THE PARSER (The Math Part)
fn parse_csi_line(line: &str) -> Option<Vec<f64>> {
    // We expect a format like: "CSI,imag1,real1,imag2,real2,..."
    // We split by comma
    let parts: Vec<&str> = line.split(',').collect();
    let mut amplitudes = Vec::new();

    // We start at index 1 to skip the "CSI" tag
    let mut i = 1;

    // Safety check: ensure we have enough parts
    if parts.len() < 2 { return None; }

    while i < parts.len() - 1 {
        // Parse two numbers at a time (Imaginary and Real)
        if let (Ok(imag), Ok(real)) = (parts[i].trim().parse::<f64>(), parts[i+1].trim().parse::<f64>()) {
            // CALCULATE MAGNITUDE: sqrt(real^2 + imag^2)
            // This converts complex signal info into "Loudness" we can plot
            let amplitude = (real.powi(2) + imag.powi(2)).sqrt();
            amplitudes.push(amplitude);
        }
        i += 2; // Jump to next pair
    }

    if amplitudes.is_empty() { None } else { Some(amplitudes) }
}
