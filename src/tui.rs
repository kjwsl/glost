use crate::Language;
use crate::glossary::ExpressionEntry;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::collections::HashSet;
use std::io;

struct App {
    entries: Vec<ExpressionEntry>,
    state: ListState,
    discarded: HashSet<usize>,
    known: HashSet<usize>,
    lang: Language,
}

impl App {
    fn new(entries: Vec<ExpressionEntry>, lang: Language) -> App {
        let mut state = ListState::default();
        if !entries.is_empty() {
            state.select(Some(0));
        }
        App {
            entries,
            state,
            discarded: HashSet::new(),
            known: HashSet::new(),
            lang,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.entries.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn toggle_discard(&mut self) {
        if let Some(i) = self.state.selected() {
            if self.discarded.contains(&i) {
                self.discarded.remove(&i);
            } else {
                self.discarded.insert(i);
                self.known.remove(&i);
            }
        }
    }

    fn toggle_known(&mut self) {
        if let Some(i) = self.state.selected() {
            if self.known.contains(&i) {
                self.known.remove(&i);
            } else {
                self.known.insert(i);
                self.discarded.insert(i);
            }
        }
    }
}

pub fn run_tui(
    entries: Vec<ExpressionEntry>,
    lang: Language,
) -> Result<(Vec<ExpressionEntry>, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    if entries.is_empty() {
        return Ok((vec![], vec![]));
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(entries, lang);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        return Err(Box::new(err));
    }

    let mut kept_entries = Vec::new();
    let mut known_words = Vec::new();

    for (i, entry) in app.entries.into_iter().enumerate() {
        if app.known.contains(&i) {
            known_words.push(entry.expression.clone());
        }
        if !app.discarded.contains(&i) {
            kept_entries.push(entry);
        }
    }

    Ok((kept_entries, known_words))
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                .split(f.area());

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                .split(chunks[0]);

            // List of expressions
            let items: Vec<ListItem> = app
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let mut style = Style::default();
                    let mut prefix = " [ ] ".to_string();

                    if app.known.contains(&i) {
                        style = style.fg(Color::Green);
                        prefix = " [K] ".to_string();
                    } else if app.discarded.contains(&i) {
                        style = style.fg(Color::Red);
                        prefix = " [X] ".to_string();
                    }

                    ListItem::new(format!(
                        "{}{} ({})",
                        prefix, entry.expression, entry.frequency
                    ))
                    .style(style)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" Expressions ({}) ", app.lang)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, main_chunks[0], &mut app.state);

            // Rich preview
            let selected_idx = app.state.selected().unwrap_or(0);
            let entry = &app.entries[selected_idx];

            let mut preview_text = vec![
                Line::from(vec![
                    Span::styled(
                        "Expression: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        &entry.expression,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ({})", entry.pos)),
                ]),
                Line::from(""),
            ];

            // CEFR Level Tag
            if let Some(cefr) = &entry.cefr_level {
                preview_text.push(Line::from(vec![
                    Span::styled("CEFR: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        cefr,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            }

            // Grammar Note
            if let Some(grammar) = &entry.grammar_note {
                preview_text.push(Line::from(vec![
                    Span::styled("Grammar: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(grammar, Style::default().fg(Color::Green)),
                ]));
            }

            preview_text.push(Line::from(""));
            preview_text.push(Line::from(Span::styled(
                "Meaning:",
                Style::default().add_modifier(Modifier::BOLD),
            )));

            // Format meanings (handling merged POS if present)
            if entry.meaning.contains(" | ") {
                for part in entry.meaning.split(" | ") {
                    preview_text.push(Line::from(format!(" • {}", part.replace("*", ""))));
                }
            } else {
                preview_text.push(Line::from(format!(" • {}", entry.meaning.replace("*", ""))));
            }

            preview_text.push(Line::from(""));
            preview_text.push(Line::from(Span::styled(
                "Context:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            preview_text.push(Line::from(format!(
                " \"{}\"",
                entry.context.as_deref().unwrap_or("No context available.")
            )));

            let preview = Paragraph::new(preview_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Analysis Preview "),
                )
                .wrap(Wrap { trim: true });
            f.render_widget(preview, main_chunks[1]);

            // Help bar
            let help = Paragraph::new(
                " [Space] Toggle Keep | [t] Mark Known | [Enter] Generate | [q] Quit ",
            )
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Char(' ') => app.toggle_discard(),
                KeyCode::Char('t') => app.toggle_known(),
                KeyCode::Enter => return Ok(()),
                _ => {}
            }
        }
    }
}
