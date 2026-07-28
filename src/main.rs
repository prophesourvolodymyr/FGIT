use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use fgit::{app::App, command::parse_args};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io::{self, IsTerminal},
    time::Duration,
};

fn main() -> Result<()> {
    let options = match parse_args(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("fgit: {error}");
            std::process::exit(2);
        }
    };
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        eprintln!("fgit: an interactive terminal is required");
        std::process::exit(2);
    }
    let mut session = TerminalSession::new()?;
    let mut app = App::loading_with_options(options);
    session.terminal.draw(|frame| app.draw(frame))?;
    app.load_repository(&std::env::current_dir()?);
    loop {
        app.tick_animation();
        session.terminal.draw(|frame| app.draw(frame))?;
        app.process_pending();
        if event::poll(Duration::from_millis(150))? && app.handle_event(event::read()?) {
            break;
        }
    }
    session.cleanup()?;
    Ok(())
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    raw_mode: bool,
    mouse_capture: bool,
    alternate_screen: bool,
    cleaned: bool,
}

impl TerminalSession {
    fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide) {
            let _ = terminal::disable_raw_mode();
            return Err(error.into());
        }
        let backend = CrosstermBackend::new(stdout);
        let terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => {
                let mut stdout = io::stdout();
                let _ = execute!(stdout, Show, DisableMouseCapture, LeaveAlternateScreen);
                let _ = terminal::disable_raw_mode();
                return Err(error.into());
            }
        };
        Ok(Self {
            terminal,
            raw_mode: true,
            mouse_capture: true,
            alternate_screen: true,
            cleaned: false,
        })
    }

    fn cleanup(&mut self) -> io::Result<()> {
        if self.cleaned {
            return Ok(());
        }
        let _ = self.terminal.show_cursor();
        let mut first_error = None;
        if let Err(error) = execute!(
            self.terminal.backend_mut(),
            Show,
            DisableMouseCapture,
            LeaveAlternateScreen
        ) {
            first_error = Some(error);
        }
        self.mouse_capture = false;
        self.alternate_screen = false;
        if self.raw_mode {
            if let Err(error) = terminal::disable_raw_mode() {
                first_error.get_or_insert(error);
            }
            self.raw_mode = false;
        }
        self.cleaned = true;
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
