mod app;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. SETUP TERMINAL
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. INITIALIZE APP
    let mut app = App::new();

    // 3. MAIN LOOP
    let res = run_app(&mut terminal, &mut app).await;

    // 4. CLEANUP
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        // --- SIMULATE INCOMING DATA ---
        // In the next step, we will replace this line with the REAL serial reader.
        // For now, we "push" data into the struct to test the connection.
        // let fake_signal = 50.0 + (app.counter as f64 / 5.0).sin() * 20.0;
        // app.push_data(fake_signal);
		// --- SIMULATE INCOMING DATA ---
        // Create a wave shape across 64 subcarriers
        let wave_data: Vec<f64> = (0..64)
            .map(|i| {
                let x = i as f64;
                // Complex wave math: varying frequency and offset based on time
                ((x / 8.0) + (app.counter as f64 / 5.0)).sin() * 20.0 + 50.0
            })
            .collect();

        app.push_data(wave_data); // Send the whole wave to storage
        // ------------------------------

        terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
                .split(area);

            // -- PREPARE DATA FROM APP STORAGE --
            // Now we read from csi_history instead of making up math here!
            let data: Vec<(f64, f64)> = match app.csi_history.back() {
                Some(packet) => packet.amplitude
                    .iter()
                    .enumerate()
                    .map(|(i, &amp)| (i as f64, amp))
                    .collect(),
                None => vec![], // Draw nothing if no data yet
            };

            let datasets = vec![
                Dataset::default()
                    .name("Real-Time CSI Storage")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Yellow)) // Changed color to Yellow
                    .data(&data),
            ];

            // -- DRAW CHART --
            let chart = Chart::new(datasets)
                .block(Block::default().title(" App Storage View ").borders(Borders::ALL))
                .x_axis(Axis::default().bounds([0.0, 64.0]))
                .y_axis(Axis::default().bounds([0.0, 100.0]));

            f.render_widget(chart, chunks[0]);
        })?;

        // INPUT HANDLING
        if event::poll(Duration::from_millis(50))? {
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
