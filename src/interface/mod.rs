use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Paragraph, Widget},
    Terminal,
};

mod field;
mod setup;

use crate::App;

use self::setup::SetupView;

pub fn run_ui(app: Arc<App>) -> Result<(), io::Error> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    let events = run_event_loop();

    loop {
        let mut view = app.current_view.lock().unwrap();

        let draw_result = terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Percentage(100)])
                .horizontal_margin(1)
                .split(f.size());

            let logo = Paragraph::new(LOGO).alignment(Alignment::Center);

            f.render_widget(logo, chunks[0]);
            f.render_widget(&*view, chunks[1]);
        });

        if let Err(err) = draw_result {
            eprintln!("Failed to draw: {:?}", err);
            break;
        };

        if let Ok(event) = events.try_recv() {
            if let Event::Key(key) = &event {
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    break;
                }
            }

            view.handle_event(event);
        };
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_event_loop() -> Receiver<Event> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || loop {
        match read() {
            Ok(event) => sender.send(event).expect("Send"),
            Err(err) => eprintln!("{:?}", err),
        };
    });

    receiver
}

pub enum View {
    Setup(SetupView),
    Dashboard,
}

impl Default for View {
    fn default() -> Self {
        Self::Setup(Default::default())
    }
}

pub trait ViewController {
    fn handle_event(&mut self, event: Event);
}

impl ViewController for View {
    fn handle_event(&mut self, event: Event) {
        match self {
            Self::Setup(setup_view) => setup_view.handle_event(event),
            Self::Dashboard => todo!(),
        }
    }
}

impl Widget for &View {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        match self {
            View::Setup(setup_view) => setup_view.render(area, buf),
            View::Dashboard => todo!(),
        }
    }
}

const LOGO: &str = "
█▀█ █░█ █░░ █▀ █▀▀ █▀ █░█ █ ▀█▀ ▀█▀ █▀▀ █▀█
█▀▀ █▄█ █▄▄ ▄█ ██▄ ▄█ █▀█ █ ░█░ ░█░ ██▄ █▀▄
";
