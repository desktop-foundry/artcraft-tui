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
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

enum Mode {
    Normal,
    Confirm {
        plan_text: String,
        plan: artcraft_domain::ReconciliationPlan,
    },
    Result {
        summary: String,
    },
}

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
    let mut mode = Mode::Normal;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(2),
                ])
                .split(f.area());
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    "artcraft-tui",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
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
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!(
                        "{} {}  {}",
                        mark,
                        a.display_name,
                        if a.is_community { "(community)" } else { "" }
                    ))
                    .style(style)
                })
                .collect();
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Applications "),
            );
            f.render_widget(list, chunks[1]);

            let footer_text = match mode {
                Mode::Normal => "q: quit  ↑/↓: navigate  a: apply (confirm)  d: discover",
                Mode::Confirm { .. } => "e: execute  c/Esc: cancel",
                Mode::Result { .. } => "any key: back to normal",
            };
            let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[2]);

            // Modal overlay
            match &mode {
                Mode::Confirm { plan_text, .. } => {
                    let area = centered_rect(60, 60, f.area());
                    f.render_widget(Clear, area);
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(" Confirm Reconciliation ");
                    let para = Paragraph::new(plan_text.clone()).block(block);
                    f.render_widget(para, area);
                }
                Mode::Result { summary } => {
                    let area = centered_rect(60, 60, f.area());
                    f.render_widget(Clear, area);
                    let block = Block::default().borders(Borders::ALL).title(" Result ");
                    let para = Paragraph::new(summary.clone()).block(block);
                    f.render_widget(para, area);
                }
                Mode::Normal => {}
            }
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match &mut mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Down => {
                            if selected + 1 < apps.len() {
                                selected += 1;
                            }
                        }
                        KeyCode::Up => {
                            selected = selected.saturating_sub(1);
                        }
                        KeyCode::Char('a') => {
                            let plan = ctx.reconcile_plan();
                            let text = artcraft_reconcile::plan_to_text(&plan);
                            mode = Mode::Confirm {
                                plan_text: text,
                                plan,
                            };
                        }
                        _ => {}
                    },
                    Mode::Confirm { plan, .. } => match key.code {
                        KeyCode::Char('e') => {
                            let results = ctx.execute_plan(plan);
                            let mut summary = String::from("Execution summary:\n");
                            for (id, ok, msg) in &results {
                                summary.push_str(&format!(
                                    "  [{}] {} - {}\n",
                                    if *ok { "OK" } else { "FAIL" },
                                    id,
                                    msg
                                ));
                            }
                            mode = Mode::Result { summary };
                        }
                        KeyCode::Char('c') | KeyCode::Esc => {
                            mode = Mode::Normal;
                        }
                        _ => {}
                    },
                    Mode::Result { .. } => {
                        mode = Mode::Normal;
                    }
                }
            }
        }
    }
    Ok(())
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
