use crate::Language;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::collections::HashSet;
use std::io;

struct App {
    words: Vec<(String, (usize, Option<String>))>,
    state: ListState,
    discarded: HashSet<usize>,
    known: HashSet<usize>,
    lang: Language,
}

impl App {
    fn new(mut words: Vec<(String, (usize, Option<String>))>, lang: Language) -> App {
        // Sort by frequency descending for the TUI
        words.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
        let mut state = ListState::default();
        if !words.is_empty() {
            state.select(Some(0));
        }
        App {
            words,
            state,
            discarded: HashSet::new(),
            known: HashSet::new(),
            lang,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.words.len() - 1 {
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
                    self.words.len() - 1
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
    words: Vec<(String, (usize, Option<String>))>,
    lang: Language,
) -> Result<(Vec<(String, (usize, Option<String>))>, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    if words.is_empty() {
        return Ok((vec![], vec![]));
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(words, lang);
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

    let mut kept_words = Vec::new();
    let mut known_words = Vec::new();

    for (i, word_data) in app.words.into_iter().enumerate() {
        if app.known.contains(&i) {
            known_words.push(word_data.0.clone());
        }
        if !app.discarded.contains(&i) {
            kept_words.push(word_data);
        }
    }

    Ok((kept_words, known_words))
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

            // List of words
            let items: Vec<ListItem> = app
                .words
                .iter()
                .enumerate()
                .map(|(i, (word, (freq, _)))| {
                    let mut style = Style::default();
                    let mut prefix = " [ ] ".to_string();

                    if app.known.contains(&i) {
                        style = style.fg(Color::Green);
                        prefix = " [K] ".to_string();
                    } else if app.discarded.contains(&i) {
                        style = style.fg(Color::Red);
                        prefix = " [X] ".to_string();
                    }

                    ListItem::new(format!("{}{} ({})", prefix, word, freq)).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!(" Words ({}) ", app.lang)))
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, main_chunks[0], &mut app.state);

            // Context preview
            let selected_word_idx = app.state.selected().unwrap_or(0);
            let context = app.words[selected_word_idx].1 .1.as_deref().unwrap_or("No context available.");
            let word = &app.words[selected_word_idx].0;

            let preview = Paragraph::new(format!("Word: {}\n\nContext:\n\n{}", word, context))
                .block(Block::default().borders(Borders::ALL).title(" Preview "))
                .wrap(Wrap { trim: true });
            f.render_widget(preview, main_chunks[1]);

            // Help bar
            let help = Paragraph::new(" [Space] Toggle Keep | [t] Mark Known | [Enter] Generate | [q] Quit ")
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
