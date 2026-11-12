use artcraft_application::AppContext;
use artcraft_domain::Application;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

pub fn run(ctx: AppContext) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let res = run_loop(&mut terminal, ctx);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ctx: AppContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut selected: usize = 0;
    let apps: Vec<Application> = ctx.catalog.clone();
    let insts = ctx.discover_installations();
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(2)])
                .split(f.area());
            let header = Paragraph::new(Line::from(vec![
                Span::styled("artcraft-tui", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("installation reconciler", Style::default().fg(Color::Gray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" ArtCraft "));
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = apps
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    let installed = insts.iter().any(|x| x.app_id == a.id);
                    let mark = if installed { "●" } else { "○" };
                    let style = if i == selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!("{} {}  {}", mark, a.display_name, if a.is_community { "(community)" } else { "" }))
                        .style(style)
                })
                .collect();
            let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Applications "));
            f.render_widget(list, chunks[1]);

            let footer = Paragraph::new("q: quit  ↑/↓: navigate  d: discover  (read-only preview)")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down => {
                        if selected + 1 < apps.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
