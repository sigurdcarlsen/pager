use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    // subtract top+bottom border (1 each) and the status bar (1)
    let page_height = area.height.saturating_sub(3) as usize;

    let constraints: Vec<Constraint> = (0..app.columns)
        .map(|_| Constraint::Ratio(1, app.columns as u32))
        .collect();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(Rect { height: area.height.saturating_sub(1), ..area });

    for (col_idx, &col_area) in cols.iter().enumerate() {
        let raw_lines = app.column_lines(col_idx, page_height);
        let text_lines: Vec<Line> = raw_lines.iter().map(|s| parse_ansi(s)).collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" {} ", col_idx + 1));

        let para = Paragraph::new(text_lines).block(block);
        frame.render_widget(para, col_area);
    }

    draw_statusbar(frame, app, page_height, area);
}

fn draw_statusbar(frame: &mut Frame, app: &App, page_height: usize, area: Rect) {
    let total = app.lines.len();
    let visible_end = (app.offset + app.columns * page_height).min(total);
    let pct = (visible_end * 100).checked_div(total).unwrap_or(100);

    let status_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    let msg = format!(
        " lines {}-{}/{} {}%  cols:{} mode:{} \
         j/k scroll · f/b page · +/- cols · v delta view · r raw · x fancy · q quit ",
        app.offset + 1,
        visible_end,
        total,
        pct,
        app.columns,
        app.formatter.label(),
    );

    let status = Paragraph::new(msg)
        .style(Style::default().fg(Color::Black).bg(Color::White));
    frame.render_widget(status, status_area);
}

/// Parse a string containing ANSI SGR escape sequences into a ratatui Line.
fn parse_ansi(s: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut style = Style::default();
    let mut remaining = s;

    loop {
        match remaining.find('\x1b') {
            None => {
                if !remaining.is_empty() {
                    spans.push(Span::styled(remaining.to_owned(), style));
                }
                break;
            }
            Some(pos) => {
                if pos > 0 {
                    spans.push(Span::styled(remaining[..pos].to_owned(), style));
                }
                remaining = &remaining[pos..];
                if remaining.starts_with("\x1b[") {
                    let rest = &remaining[2..];
                    if let Some(end) = rest.find('m') {
                        style = apply_sgr(style, &rest[..end]);
                        remaining = &rest[end + 1..];
                    } else {
                        // malformed sequence — skip the ESC
                        remaining = &remaining[1..];
                    }
                } else {
                    remaining = &remaining[1..];
                }
            }
        }
    }

    Line::from(spans)
}

fn apply_sgr(mut style: Style, codes: &str) -> Style {
    if codes.is_empty() {
        return Style::reset();
    }
    let parts: Vec<&str> = codes.split(';').collect();
    let mut i = 0;
    while i < parts.len() {
        let code: u8 = parts[i].parse().unwrap_or(0);
        match code {
            0 => style = Style::reset(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            7 => style = style.add_modifier(Modifier::REVERSED),
            9 => style = style.add_modifier(Modifier::CROSSED_OUT),
            22 => style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            27 => style = style.remove_modifier(Modifier::REVERSED),
            30 => style = style.fg(Color::Black),
            31 => style = style.fg(Color::Red),
            32 => style = style.fg(Color::Green),
            33 => style = style.fg(Color::Yellow),
            34 => style = style.fg(Color::Blue),
            35 => style = style.fg(Color::Magenta),
            36 => style = style.fg(Color::Cyan),
            37 => style = style.fg(Color::White),
            38 if i + 2 < parts.len() && parts[i + 1] == "5" => {
                if let Ok(n) = parts[i + 2].parse::<u8>() {
                    style = style.fg(Color::Indexed(n));
                }
                i += 2;
            }
            38 if i + 4 < parts.len() && parts[i + 1] == "2" => {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[i + 2].parse::<u8>(),
                    parts[i + 3].parse::<u8>(),
                    parts[i + 4].parse::<u8>(),
                ) {
                    style = style.fg(Color::Rgb(r, g, b));
                }
                i += 4;
            }
            39 => style = style.fg(Color::Reset),
            40 => style = style.bg(Color::Black),
            41 => style = style.bg(Color::Red),
            42 => style = style.bg(Color::Green),
            43 => style = style.bg(Color::Yellow),
            44 => style = style.bg(Color::Blue),
            45 => style = style.bg(Color::Magenta),
            46 => style = style.bg(Color::Cyan),
            47 => style = style.bg(Color::White),
            48 if i + 2 < parts.len() && parts[i + 1] == "5" => {
                if let Ok(n) = parts[i + 2].parse::<u8>() {
                    style = style.bg(Color::Indexed(n));
                }
                i += 2;
            }
            48 if i + 4 < parts.len() && parts[i + 1] == "2" => {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[i + 2].parse::<u8>(),
                    parts[i + 3].parse::<u8>(),
                    parts[i + 4].parse::<u8>(),
                ) {
                    style = style.bg(Color::Rgb(r, g, b));
                }
                i += 4;
            }
            49 => style = style.bg(Color::Reset),
            90 => style = style.fg(Color::DarkGray),
            91 => style = style.fg(Color::LightRed),
            92 => style = style.fg(Color::LightGreen),
            93 => style = style.fg(Color::LightYellow),
            94 => style = style.fg(Color::LightBlue),
            95 => style = style.fg(Color::LightMagenta),
            96 => style = style.fg(Color::LightCyan),
            97 => style = style.fg(Color::Gray),
            100 => style = style.bg(Color::DarkGray),
            101 => style = style.bg(Color::LightRed),
            102 => style = style.bg(Color::LightGreen),
            103 => style = style.bg(Color::LightYellow),
            104 => style = style.bg(Color::LightBlue),
            105 => style = style.bg(Color::LightMagenta),
            106 => style = style.bg(Color::LightCyan),
            107 => style = style.bg(Color::Gray),
            _ => {}
        }
        i += 1;
    }
    style
}
