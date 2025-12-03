mod app;
mod csi_reader; // <--- Import the new reader file

use std::{io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Terminal,
};
use app::App;
use tokio::sync::mpsc; // <--- For the communication channel

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. SETUP TERMINAL
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. INITIALIZE APP & CHANNEL
    let mut app = App::new();
    // Create a channel: tx (transmitter) goes to the reader, rx (receiver) stays here
    let (tx, mut rx) = mpsc::channel(100);

    // 3. SPAWN THE LISTENER (Background Task)
    tokio::spawn(async move {
        csi_reader::run_listener(tx).await;
    });

    // 4. MAIN LOOP
    let res = run_app(&mut terminal, &mut app, &mut rx).await;

    // 5. CLEANUP
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: &mut mpsc::Receiver<Vec<f64>> // <--- Receiver passed in here
) -> io::Result<()> {

    loop {
        // --- READ REAL DATA ---
        // Check if any new data arrived from the ESP32
        while let Ok(real_data) = rx.try_recv() {
             app.push_data(real_data);
        }
        // ----------------------

        terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
                .split(area);

            // -- PREPARE DATA FOR CHART --
            let data: Vec<(f64, f64)> = match app.csi_history.back() {
                Some(packet) => packet.amplitude
                    .iter()
                    .enumerate()
                    .map(|(i, &amp)| (i as f64, amp))
                    .collect(),
                None => vec![],
            };

            let datasets = vec![
                Dataset::default()
                    .name("Real-Time CSI Data")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&data),
            ];

            // -- DRAW CHART --
            let chart = Chart::new(datasets)
                .block(Block::default().title(" Live WiFi CSI ").borders(Borders::ALL))
                .x_axis(Axis::default().bounds([0.0, 64.0]).title("Subcarriers"))
                .y_axis(Axis::default().bounds([0.0, 60.0]).title("Amplitude")); // Adjusted scale to 60 for typical CSI

            f.render_widget(chart, chunks[0]);
        })?;

        // INPUT HANDLING
        if event::poll(Duration::from_millis(16))? { // 60 FPS
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    app.quit();
                }
            }
        }

        app.on_tick();

        if app.should_quit {
            return Ok(());
        }
    }
}
