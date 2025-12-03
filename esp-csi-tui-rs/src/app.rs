use std::collections::VecDeque;

// This struct holds the data for one "frame" of WiFi CSI
pub struct CsiPacket {
    pub amplitude: Vec<f64>, // The "loudness" of the signal
    pub timestamp: u64,
}

pub struct App {
    pub should_quit: bool,
    pub counter: u64,
    // Store the last 100 packets for the graph
    pub csi_history: VecDeque<CsiPacket>,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            counter: 0,
            csi_history: VecDeque::with_capacity(100),
        }
    }

    pub fn on_tick(&mut self) {
        self.counter += 1;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    // Call this when we get a new line of text from the ESP32
    // pub fn push_data(&mut self, fake_amplitude: f64) {
    //     // In the real version, we will parse the CSV string here.
    //     // For now, we simulate data to test the chart.
    //     if self.csi_history.len() >= 100 {
    //         self.csi_history.pop_front();
    //     }

    //     // Creating fake data just to test the UI
    //     self.csi_history.push_back(CsiPacket {
    //         amplitude: vec![fake_amplitude; 64], // 64 subcarriers is standard
    //         timestamp: self.counter,
    //     });
    // }
	// Update this function to take a whole Vector of data
    pub fn push_data(&mut self, incoming_data: Vec<f64>) {
        if self.csi_history.len() >= 100 {
            self.csi_history.pop_front();
        }

        self.csi_history.push_back(CsiPacket {
            amplitude: incoming_data, // Use the list we passed in
            timestamp: self.counter,
        });
    }
}
