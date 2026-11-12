use artcraft_application::AppContext;
use artcraft_domain::{Application, ApplicationId, Platform};

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
use std::collections::HashSet;
use std::io;

enum Mode {
    Normal,
    Detail {
        index: usize,
    },
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

fn estimate_size(app_id: &ApplicationId, platform: &Platform) -> String {
    // Imagined estimates; to be replaced by an up-to-date database.
    let base = match app_id.0.as_str() {
        "photocraft" => 180,
        "vectorcraft" => 120,
        "filmcraft" => 420,
        "lightcraft" => 260,
        "pdfcraft" => 95,
        "effectcraft" => 310,
        "anicraft" => 150,
        "designcraft" => 210,
        "figcraft" => 190,
        "texturecraft" => 380,
        "materialcraft" => 220,
        "wordcraft" => 160,
        "soundcraft" => 290,
        "deckcraft" => 140,
        "cadcraft" => 330,
        "gridcraft" => 170,
        "craft-anything" => 200,
        "artcraft-launcher" => 60,
        "solidcraft" => 240,
        "mailcraft" => 130,
        _ => 150,
    };
    let mb = match platform {
        Platform::Linux => base,
        Platform::MacOS => (base as f64 * 1.05) as i32,
        Platform::Windows => (base as f64 * 1.1) as i32,
        Platform::Unknown => base,
    };
    if mb >= 1024 {
        format!("{:.1} GB", mb as f64 / 1024.0)
    } else {
        format!("{} MB", mb)
    }
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ctx: AppContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut selected: usize = 0;
    let apps: Vec<Application> = ctx.catalog.clone();
    let insts = ctx.discover_installations();
    let mut mode = Mode::Normal;
    let mut to_install: HashSet<ApplicationId> = HashSet::new();
    let mut to_remove: HashSet<ApplicationId> = HashSet::new();
    if let Some(store) = &ctx.event_store {
        for app in store.load_pending().unwrap_or_default() {
            to_install.insert(ApplicationId::new(app));
        }
    }

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.area());
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    "artcraft",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled("installer", Style::default().fg(Color::Gray)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(" ArtCraft "));
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = apps
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    let installed = insts.iter().any(|x| x.app_id == a.id);
                    let mark = if installed { "●" } else { "○" };
                    let checked = if to_install.contains(&a.id) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    let size = estimate_size(&a.id, &ctx.platform.os);
                    let style = if i == selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!(
                        "{} {} {:<18} {}  {}",
                        checked,
                        mark,
                        a.display_name,
                        if a.is_community {
                            "(community)"
                        } else {
                            "           "
                        },
                        size
                    ))
                    .style(style)
                })
                .collect();
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Applications (Space/Enter to select) "),
            );
            f.render_widget(list, chunks[1]);

            let footer_text = match mode {
                Mode::Normal => "q: quit  ↑/↓: navigate  Space: select  Enter: details  a: apply",
                Mode::Detail { .. } => "i: install  u: upgrade  r: remove  Esc: back",
                Mode::Confirm { .. } => "e: execute  c/Esc: cancel",
                Mode::Result { .. } => "any key: back",
            };
            let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[2]);

            match &mode {
                Mode::Detail { index } => {
                    if let Some(app) = apps.get(*index) {
                        let installed = insts.iter().any(|x| x.app_id == app.id);
                        let inst_ver = insts
                            .iter()
                            .find(|x| x.app_id == app.id)
                            .and_then(|x| x.installed_version.clone())
                            .unwrap_or_else(|| "—".into());
                        let latest = app
                            .latest_version
                            .clone()
                            .unwrap_or_else(|| "unknown".into());
                        let status = if installed {
                            "Installed"
                        } else {
                            "Not installed"
                        };
                        let size = estimate_size(&app.id, &ctx.platform.os);
                        let mut text = String::new();
                        text.push_str(&format!(
                            "Name: {}
",
                            app.display_name
                        ));
                        text.push_str(&format!(
                            "Installed version: {}
",
                            inst_ver
                        ));
                        text.push_str(&format!(
                            "Latest version: {}
",
                            latest
                        ));
                        text.push_str(&format!(
                            "Description: {}
",
                            app.short_description
                        ));
                        text.push_str(&format!(
                            "Size once installed: {}
",
                            size
                        ));
                        text.push_str(&format!(
                            "Status: {}

",
                            status
                        ));
                        text.push_str(
                            "Actions:
",
                        );
                        text.push_str(
                            "  i - Install
",
                        );
                        text.push_str(
                            "  u - Upgrade
",
                        );
                        text.push_str(
                            "  r - Remove
",
                        );
                        text.push_str(
                            "  Esc - Back
",
                        );
                        let area = centered_rect(60, 60, f.area());
                        f.render_widget(Clear, area);
                        let block = Block::default()
                            .borders(Borders::ALL)
                            .title(" Application Details ");
                        let para = Paragraph::new(text).block(block);
                        f.render_widget(para, area);
                    }
                }
                Mode::Confirm { plan_text, .. } => {
                    let area = centered_rect(70, 70, f.area());
                    f.render_widget(Clear, area);
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(" Confirm Install ");
                    let para = Paragraph::new(plan_text.clone()).block(block);
                    f.render_widget(para, area);
                }
                Mode::Result { summary } => {
                    let area = centered_rect(70, 70, f.area());
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
                        KeyCode::Char(' ') => {
                            if let Some(app) = apps.get(selected) {
                                if to_install.contains(&app.id) {
                                    to_install.remove(&app.id);
                                } else {
                                    to_install.insert(app.id.clone());
                                }
                            }
                            if let Some(store) = &ctx.event_store {
                                let sel: Vec<String> =
                                    to_install.iter().map(|x| x.0.clone()).collect();
                                let _ = store.save_pending(&sel);
                            }
                        }
                        KeyCode::Enter => {
                            mode = Mode::Detail { index: selected };
                        }
                        KeyCode::Char('a') => {
                            if let Some(store) = &ctx.event_store {
                                let sel: Vec<String> =
                                    to_install.iter().map(|x| x.0.clone()).collect();
                                let _ = store.save_pending(&sel);
                            }
                            // build desired state from selection
                            let mut cfg = artcraft_config::AppConfig::load().unwrap_or_default();
                            let desired: Vec<String> =
                                to_install.iter().map(|x| x.0.clone()).collect();
                            cfg.desired_apps = desired;
                            let _ = cfg.save();
                            let plan = ctx.reconcile_plan();
                            let text = artcraft_reconcile::plan_to_text(&plan);
                            mode = Mode::Confirm {
                                plan_text: text,
                                plan,
                            };
                        }
                        _ => {}
                    },
                    Mode::Detail { index } => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            mode = Mode::Normal;
                        }
                        KeyCode::Char('i') => {
                            if let Some(app) = apps.get(*index) {
                                if !to_install.contains(&app.id) {
                                    to_install.insert(app.id.clone());
                                }
                                to_remove.remove(&app.id);
                            }
                            mode = Mode::Normal;
                        }
                        KeyCode::Char('u') => {
                            if let Some(app) = apps.get(*index) {
                                if !to_install.contains(&app.id) {
                                    to_install.insert(app.id.clone());
                                }
                            }
                            mode = Mode::Normal;
                        }
                        KeyCode::Char('r') => {
                            if let Some(app) = apps.get(*index) {
                                if !to_remove.contains(&app.id) {
                                    to_remove.insert(app.id.clone());
                                }
                                to_install.remove(&app.id);
                            }
                            mode = Mode::Normal;
                        }
                        _ => {}
                    },
                    Mode::Confirm { plan, .. } => match key.code {
                        KeyCode::Char('e') => {
                            if let Some(store) = &ctx.event_store {
                                let _ = store.clear_pending();
                            }
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
