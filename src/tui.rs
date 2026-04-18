use crate::Language;
use crate::glossary::ExpressionEntry;
use anyhow::Result;
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
use std::io;

#[derive(Debug, Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Color {
        Color::Rgb(rgb.r, rgb.g, rgb.b)
    }
}

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        Rgb {
            r: $r,
            g: $g,
            b: $b,
        }
    };
}

#[allow(unused)]
#[derive(Debug)]
struct AppTheme {
    rosewater: Rgb,
    flamingo: Rgb,
    pink: Rgb,
    mauve: Rgb,
    red: Rgb,
    maroon: Rgb,
    peach: Rgb,
    yellow: Rgb,
    green: Rgb,
    teal: Rgb,
    sky: Rgb,
    sapphire: Rgb,
    blue: Rgb,
    lavender: Rgb,
    text: Rgb,
    subtext1: Rgb,
    subtext0: Rgb,
    overlay2: Rgb,
    overlay1: Rgb,
    overlay0: Rgb,
    surface2: Rgb,
    surface1: Rgb,
    surface0: Rgb,
    base: Rgb,
    mantle: Rgb,
    crust: Rgb,
}

const CATPPUCCIN_MOCHA: AppTheme = AppTheme {
    rosewater: rgb!(245, 224, 220),
    flamingo: rgb!(242, 205, 205),
    pink: rgb!(245, 194, 231),
    mauve: rgb!(203, 166, 247),
    red: rgb!(243, 139, 168),
    maroon: rgb!(235, 160, 172),
    peach: rgb!(250, 179, 135),
    yellow: rgb!(249, 226, 175),
    green: rgb!(166, 227, 161),
    teal: rgb!(148, 226, 213),
    sky: rgb!(137, 220, 235),
    sapphire: rgb!(116, 199, 236),
    blue: rgb!(137, 180, 250),
    lavender: rgb!(180, 190, 254),
    text: rgb!(205, 214, 244),
    subtext1: rgb!(186, 194, 222),
    subtext0: rgb!(166, 173, 200),
    overlay2: rgb!(147, 153, 178),
    overlay1: rgb!(127, 132, 156),
    overlay0: rgb!(108, 112, 134),
    surface2: rgb!(88, 91, 112),
    surface1: rgb!(69, 71, 90),
    surface0: rgb!(49, 50, 68),
    base: rgb!(30, 30, 46),
    mantle: rgb!(24, 24, 37),
    crust: rgb!(17, 17, 27),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum EntryMarker {
    #[default]
    None,
    /// Exclude expression from being included in the glossary
    Discarded,
    /// Same as discarded but also marked as known so they don't show up again
    Known,
}

#[derive(Debug, Clone, Default)]
struct EntryState {
    marker: EntryMarker,
    selected: bool,
    /// Highlighted in Visual mode.
    highlighted: bool,
}

#[derive(Debug)]
struct App<'a> {
    entries: &'a [ExpressionEntry],
    list: ListState,
    states: Vec<EntryState>,
    visual_anchor: Option<usize>,
    visual_mode: bool,
    lang: Language,
    theme: &'static AppTheme,
}

impl<'a> App<'a> {
    fn new(entries: &'a [ExpressionEntry], lang: Language) -> Self {
        let mut state = ListState::default();
        if !entries.is_empty() {
            state.select(Some(0));
        }
        App {
            entries,
            list: state,
            states: vec![EntryState::default(); entries.len()],
            visual_anchor: None,
            visual_mode: false,
            lang,
            theme: &CATPPUCCIN_MOCHA,
        }
    }

    fn next(&mut self) {
        let i = match self.list.selected() {
            Some(i) => {
                if i >= self.entries.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list.select(Some(i));
    }

    fn toggle_marker(&mut self, marker: EntryMarker) {
        let selected: Vec<_> = self
            .states
            .iter_mut()
            .filter(|s| s.selected | s.highlighted)
            .collect();
        if !selected.is_empty() {
            let first_marker = selected[0].marker;
            let mut target_marker = marker;
            if selected.iter().skip(1).all(|s| s.marker == first_marker) {
                target_marker = if first_marker == marker {
                    EntryMarker::None
                } else {
                    marker
                };
            }

            for state in selected {
                state.marker = target_marker;
            }
        } else if let Some(i) = self.list.selected() {
            self.states[i].marker = if self.states[i].marker == marker {
                EntryMarker::None
            } else {
                marker
            };
        }
    }

    fn toggle_selection(&mut self) {
        if let Some(i) = self.list.selected() {
            self.states[i].selected = !self.states[i].selected;
        }
    }

    fn toggle_visual_mode(&mut self) {
        self.set_visual_mode(!self.visual_mode);
    }

    fn set_visual_mode(&mut self, mode: bool) {
        if let Some(i) = self.list.selected() {
            if mode {
                self.visual_mode = true;
                self.visual_anchor = Some(i);
            } else {
                self.visual_mode = false;
                self.reset_selections();
            }
            self.update_visual_selection();
        }
    }

    fn reset_selections(&mut self) {
        for state in self.states.iter_mut() {
            state.highlighted = false;
            state.selected = false;
        }
    }

    fn update_visual_selection(&mut self) {
        if self.visual_mode {
            let anchor = self.visual_anchor.unwrap_or(0);
            let cursor = self.list.selected().unwrap_or(0);
            let range = anchor.min(cursor)..=anchor.max(cursor);
            for (i, state) in self.states.iter_mut().enumerate() {
                state.highlighted = range.contains(&i);
            }
        }
    }
}

pub fn run_tui(
    entries: &[ExpressionEntry],
    lang: Language,
) -> Result<(Vec<ExpressionEntry>, Vec<String>)> {
    if entries.is_empty() {
        return Ok((vec![], vec![]));
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(entries, lang);
    run_app(&mut terminal, &mut app)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let kept_entries: Vec<ExpressionEntry> = app
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| match app.states[i].marker {
            EntryMarker::Discarded | EntryMarker::Known => None,
            _ => Some(e.to_owned()),
        })
        .collect();
    let known_words: Vec<String> = app
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| match app.states[i].marker {
            EntryMarker::Known => Some(e.expression.to_owned()),
            _ => None,
        })
        .collect();

    Ok((kept_entries, known_words))
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let highlight_bg: Color = app.theme.lavender.into();
    let highlight_fg: Color = app.theme.base.into();

    loop {
        terminal
            .draw(|f| {
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
                        let is_selected = app.states[i].selected || app.states[i].highlighted;
                        let is_cursor = app.list.selected() == Some(i) && app.visual_mode;

                        let sel = if is_selected {
                            ">".to_string()
                        } else if is_cursor {
                            "~".to_string()
                        } else {
                            " ".to_string()
                        };

                        let mut style = Style::default();
                        let mut state = "[ ]".to_string();

                        match app.states[i].marker {
                            EntryMarker::Known => {
                            style = style.fg(app.theme.green.into());
                            state = "[K]".to_string();
                            }
                            EntryMarker::Discarded => {
                            style = style.fg(app.theme.red.into());
                            state = "[X]".to_string();
                            }
                            _ => (),
                        }

                        if app.states[i].highlighted || app.states[i].selected {
                            style = style.add_modifier(Modifier::BOLD).bg(highlight_bg).fg(highlight_fg);
                        }

                        ListItem::new(format!(
                            "{} {} {} ({})",
                            sel, state, entry.expression, entry.frequency
                        ))
                        .style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .style(Style::default().fg(app.theme.text.into()))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!(" Expressions ({}) ", app.lang)),
                    )
                    .highlight_style(
                        Style::default()
                        .fg(if app.visual_mode { highlight_fg } else { app.theme.text.into() })
                            .bg(if app.visual_mode { highlight_bg } else { app.theme.surface0.into() })
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");

                f.render_stateful_widget(list, main_chunks[0], &mut app.list);

                // Rich preview
                let selected_idx = app.list.selected().unwrap_or(0);
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
                                .fg(app.theme.rosewater.into())
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
                                .fg(app.theme.mauve.into())
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }

                // Grammar Note
                if let Some(grammar) = &entry.grammar_note {
                    preview_text.push(Line::from(vec![
                        Span::styled("Grammar: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(grammar, Style::default().fg(app.theme.green.into())),
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
                    " [Space] Select | [v] Visual | [x] Discard | [t] Known | [Esc] Exit visual | [Enter] Generate | [q] Quit ",
                )
                .block(Block::default().borders(Borders::ALL));
                f.render_widget(help, chunks[1]);
            })
            .map_err(|_| anyhow::anyhow!("Failed to render TUI"))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => app.set_visual_mode(false),
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next();
                    app.update_visual_selection();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous();
                    app.update_visual_selection();
                }
                KeyCode::Char(' ') => app.toggle_selection(),
                KeyCode::Char('v') | KeyCode::Char('V') => app.toggle_visual_mode(),
                KeyCode::Char('t') => app.toggle_marker(EntryMarker::Known),
                KeyCode::Char('x') => app.toggle_marker(EntryMarker::Discarded),
                KeyCode::Enter => return Ok(()),
                _ => {}
            }
        }
    }
}
