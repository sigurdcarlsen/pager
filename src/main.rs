mod app;
mod ui;

use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;

enum Formatter {
    None,
    Delta { side_by_side: bool },
    Fancy,
}

struct Args {
    columns: usize,
    formatter: Formatter,
}

fn parse_args() -> Args {
    let argv: Vec<String> = std::env::args().collect();
    let mut columns = 2;
    let mut formatter = Formatter::None;
    let mut it = argv.iter().skip(1).peekable();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--delta"     => formatter = Formatter::Delta { side_by_side: false },
            "--delta-sbs" => formatter = Formatter::Delta { side_by_side: true },
            "--fancy"     => formatter = Formatter::Fancy,
            "--columns" | "-n" => {
                if let Some(v) = it.next() {
                    columns = v.parse().unwrap_or(columns);
                }
            }
            s => {
                if let Ok(n) = s.parse::<usize>() {
                    columns = n;
                }
            }
        }
    }
    Args { columns, formatter }
}

fn main() -> io::Result<()> {
    let args = parse_args();

    let term_width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(220);
    let col_width = (term_width.saturating_sub(args.columns as u16 * 2)) / args.columns as u16;

    let mut stdin_bytes = Vec::new();
    io::stdin().read_to_end(&mut stdin_bytes)?;

    let content = match args.formatter {
        Formatter::None => String::from_utf8_lossy(&stdin_bytes).into_owned(),
        Formatter::Delta { side_by_side } => run_through_delta(&stdin_bytes, side_by_side, col_width)?,
        Formatter::Fancy => run_through_fancy(&stdin_bytes)?,
    };

    let lines: Vec<String> = content.lines().map(|l| l.to_owned()).collect();

    if lines.is_empty() {
        eprintln!("pager: no input");
        return Ok(());
    }

    let mut app = App::new(lines, args.columns);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn pipe_through(input: &[u8], cmd: &mut Command, install_hint: &str) -> io::Result<String> {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            io::Error::new(e.kind(), format!("failed to launch {install_hint}: {e}"))
        })?;

    child.stdin.take().unwrap().write_all(input)?;
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_through_delta(input: &[u8], side_by_side: bool, col_width: u16) -> io::Result<String> {
    let mut cmd = Command::new("delta");
    cmd.args(["--pager", "never"]);
    if side_by_side {
        cmd.arg("--side-by-side");
        cmd.args(["--width", &col_width.to_string()]);
    } else {
        // Delta has no --no-side-by-side flag, so override any global Git setting.
        let mut config = std::env::var("GIT_CONFIG_PARAMETERS").unwrap_or_default();
        if !config.is_empty() {
            config.push(' ');
        }
        config.push_str("'delta.side-by-side=false'");
        cmd.env("GIT_CONFIG_PARAMETERS", config);
    }
    pipe_through(input, &mut cmd, "delta (cargo install git-delta)")
}

fn run_through_fancy(input: &[u8]) -> io::Result<String> {
    let mut cmd = Command::new("diff-so-fancy");
    pipe_through(input, &mut cmd, "diff-so-fancy (npm install -g diff-so-fancy)")
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if !event::poll(Duration::from_millis(50))? {
            continue;
        }

        let page_height = terminal
            .size()
            .map(|s| s.height.saturating_sub(2) as usize)
            .unwrap_or(40);

        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                (KeyCode::Char('j'), _) | (KeyCode::Down, _) => app.scroll_down(page_height),
                (KeyCode::Char('k'), _) | (KeyCode::Up, _) => app.scroll_up(),
                (KeyCode::Char('d'), _) => app.page_down(page_height / 2),
                (KeyCode::Char('u'), _) => app.page_up(page_height / 2),
                (KeyCode::Char('f'), _) | (KeyCode::PageDown, _) => app.page_down(page_height),
                (KeyCode::Char('b'), _) | (KeyCode::PageUp, _) => app.page_up(page_height),
                (KeyCode::Char('g'), _) | (KeyCode::Home, _) => app.offset = 0,
                (KeyCode::Char('G'), _) | (KeyCode::End, _) => app.scroll_to_end(page_height),
                (KeyCode::Char('+'), _) | (KeyCode::Char('='), _) => {
                    app.columns = (app.columns + 1).min(8);
                }
                (KeyCode::Char('-'), _) => {
                    app.columns = app.columns.saturating_sub(1).max(1);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
