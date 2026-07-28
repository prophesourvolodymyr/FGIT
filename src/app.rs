use crate::git::{DiffPreview, FileChange, RepositoryState};
use anyhow::Result;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub enum Screen {
    Loading,
    Selection,
    Diff,
    Help,
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayoutMode {
    Small,
    Big,
    TooSmall,
}

#[derive(Default)]
struct HitAreas {
    files: Rect,
    help: Rect,
    quit: Rect,
}

pub struct App {
    pub screen: Screen,
    pub repository: Option<RepositoryState>,
    repository_path: Option<PathBuf>,
    selected: HashSet<PathBuf>,
    focused: usize,
    diffs: HashMap<PathBuf, Result<DiffPreview, String>>,
    hit_areas: HitAreas,
}

impl App {
    pub fn loading() -> Self {
        Self {
            screen: Screen::Loading,
            repository: None,
            repository_path: None,
            selected: HashSet::new(),
            focused: 0,
            diffs: HashMap::new(),
            hit_areas: HitAreas::default(),
        }
    }

    pub fn load_repository(&mut self, path: &Path) {
        self.repository_path = Some(path.to_path_buf());
        match RepositoryState::load(path) {
            Ok(repository) => {
                self.repository = Some(repository);
                self.screen = Screen::Selection;
            }
            Err(error) => {
                self.screen =
                    Screen::Error(format!("Git could not read this repository.\n\n{error}"))
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.hit_areas = HitAreas::default();
        let area = frame.area();
        if area.width < 52 || area.height < 14 {
            self.draw_too_small(frame, area);
            return;
        }
        match &self.screen {
            Screen::Loading => self.draw_loading(frame, area),
            Screen::Selection => self.draw_selection(frame, area),
            Screen::Diff => self.draw_diff(frame, area),
            Screen::Help => self.draw_help(frame, area),
            Screen::Error(message) => self.draw_error(frame, area, message),
        }
    }

    pub fn handle_event(&mut self, event: Event) -> bool {
        let Event::Key(key) = event else {
            if let Event::Mouse(mouse) = event {
                self.handle_mouse(mouse);
            }
            return false;
        };
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }
        match self.screen {
            Screen::Loading => matches!(key.code, KeyCode::Esc | KeyCode::Char('q')),
            Screen::Error(_) => {
                if key.code == KeyCode::Char('r') {
                    if let Some(path) = self.repository_path.clone() {
                        self.screen = Screen::Loading;
                        self.load_repository(&path);
                    }
                    false
                } else {
                    matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
                }
            }
            Screen::Help => {
                if matches!(
                    key.code,
                    KeyCode::Esc
                        | KeyCode::Enter
                        | KeyCode::Char('q')
                        | KeyCode::Char('b')
                        | KeyCode::Char('h')
                        | KeyCode::Char('?')
                ) {
                    self.screen = Screen::Selection;
                }
                false
            }
            Screen::Diff => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('b')) {
                    self.screen = Screen::Selection;
                }
                false
            }
            Screen::Selection => self.handle_selection_key(key),
        }
    }

    fn handle_selection_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => true,
            KeyCode::Char('h') | KeyCode::Char('?') => {
                self.screen = Screen::Help;
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.focused = self.focused.saturating_sub(1);
                false
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.focused = (self.focused + 1).min(self.file_count().saturating_sub(1));
                false
            }
            KeyCode::Char(' ') => {
                self.toggle_focused();
                false
            }
            KeyCode::Char('a') => {
                self.select_all(false);
                false
            }
            KeyCode::Char('u') => {
                self.select_all(true);
                false
            }
            KeyCode::Char('n') => {
                self.selected.clear();
                false
            }
            KeyCode::Enter => {
                if self.file_count() > 0 {
                    self.load_focused_diff();
                    self.screen = Screen::Diff;
                }
                false
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return;
        }
        if matches!(self.screen, Screen::Help | Screen::Diff) {
            self.screen = Screen::Selection;
            return;
        }
        if !matches!(self.screen, Screen::Selection) {
            return;
        }
        let point = (mouse.column, mouse.row);
        if contains(self.hit_areas.help, point) {
            self.screen = Screen::Help;
            return;
        }
        if contains(self.hit_areas.quit, point) {
            self.screen =
                Screen::Error("Quit requested. Press Esc or q to leave FGIT.".to_string());
            return;
        }
        if contains(self.hit_areas.files, point) {
            let row = mouse.row.saturating_sub(self.hit_areas.files.y + 1) as usize;
            if row < self.file_count() {
                self.focused = row;
                if mouse.column < self.hit_areas.files.x + 5 {
                    self.toggle_focused();
                }
            }
        }
    }

    fn select_all(&mut self, tracked_only: bool) {
        let Some(repository) = &self.repository else {
            return;
        };
        self.selected = repository
            .files
            .iter()
            .filter(|file| file.is_safe_to_select() && (!tracked_only || !file.is_untracked()))
            .map(|file| file.path.clone())
            .collect();
    }

    fn toggle_focused(&mut self) {
        let Some(file) = self.focused_file() else {
            return;
        };
        if !file.is_safe_to_select() {
            return;
        }
        let path = file.path.clone();
        if !self.selected.remove(&path) {
            self.selected.insert(path);
        }
    }

    fn load_focused_diff(&mut self) {
        let Some(file) = self.focused_file().cloned() else {
            return;
        };
        if self.diffs.contains_key(&file.path) {
            return;
        }
        let Some(repository) = &self.repository else {
            return;
        };
        self.diffs.insert(
            file.path.clone(),
            repository
                .diff_for(&file)
                .map_err(|error| error.to_string()),
        );
    }

    fn focused_file(&self) -> Option<&FileChange> {
        self.repository.as_ref()?.files.get(self.focused)
    }
    fn file_count(&self) -> usize {
        self.repository
            .as_ref()
            .map_or(0, |repository| repository.files.len())
    }

    fn draw_loading(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new("FGIT\n\nReading Git repository state...")
                .block(panel(" loading ", Color::Cyan))
                .centered(),
            centered(area, 58, 9),
        );
    }

    fn draw_too_small(&self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "FGIT needs at least 52 columns x 14 rows.\nCurrent terminal: {} x {}.\n\nResize the terminal, then FGIT will redraw safely.\n\nq or Esc quits.",
            area.width, area.height
        );
        frame.render_widget(
            Paragraph::new(text)
                .block(panel(" terminal too small ", Color::Yellow))
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn draw_selection(&mut self, frame: &mut Frame, area: Rect) {
        let Some(repository) = &self.repository else {
            return;
        };
        let file_count = repository.files.len();
        let branch = repository.branch.clone();
        let conflicts = repository
            .files
            .iter()
            .filter(|file| file.is_conflict())
            .count();
        let mode = layout_mode(area, file_count);
        let title = format!(
            " FGIT  {}  {} changed  {} selected ",
            branch,
            file_count,
            self.selected.len()
        );
        let outer = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);
        let inner = outer.inner(area);
        frame.render_widget(outer, area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(3)])
            .split(inner);
        match mode {
            LayoutMode::Small => {
                let body = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(43), Constraint::Percentage(57)])
                    .split(chunks[0]);
                self.draw_files(frame, body[0]);
                self.draw_preview(frame, body[1]);
            }
            LayoutMode::Big => {
                let body = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(57), Constraint::Percentage(43)])
                    .split(chunks[0]);
                self.draw_files(frame, body[0]);
                self.draw_preview(frame, body[1]);
            }
            LayoutMode::TooSmall => self.draw_too_small(frame, area),
        }
        let notice = if conflicts > 0 {
            format!(" {conflicts} conflict(s): selection is visible, but committing is blocked. ")
        } else {
            " Selection only. Staging and committing arrive in the next phases. ".to_string()
        };
        let controls = Line::from(vec![
            Span::styled(
                " Space",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" select  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" diff  "),
            Span::styled(
                "a/u/n",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" all/tracked/none  "),
            Span::styled(
                "h/?",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" help  "),
            Span::styled(
                "q",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" quit"),
        ]);
        let footer = Paragraph::new(vec![
            controls,
            Line::styled(
                notice,
                Style::default().fg(if conflicts > 0 {
                    Color::Red
                } else {
                    Color::DarkGray
                }),
            ),
        ])
        .wrap(Wrap { trim: true });
        frame.render_widget(footer, chunks[1]);
        self.hit_areas.help = Rect::new(
            chunks[1].x,
            chunks[1].y,
            chunks[1].width.saturating_sub(10),
            chunks[1].height,
        );
        self.hit_areas.quit = Rect::new(
            chunks[1].x + chunks[1].width.saturating_sub(10),
            chunks[1].y,
            10,
            chunks[1].height,
        );
    }

    fn draw_files(&mut self, frame: &mut Frame, area: Rect) {
        let Some(repository) = &self.repository else {
            return;
        };
        self.hit_areas.files = area;
        if repository.files.is_empty() {
            frame.render_widget(
                Paragraph::new("Nothing to commit.\n\nFGIT found a clean working tree.")
                    .block(panel(" changed files ", Color::Green))
                    .centered(),
                area,
            );
            return;
        }
        let items: Vec<_> = repository
            .files
            .iter()
            .map(|file| {
                let selected = self.selected.contains(&file.path);
                let marker = if selected { "[x]" } else { "[ ]" };
                let status_color = if file.is_conflict() {
                    Color::Red
                } else if file.is_untracked() {
                    Color::Yellow
                } else if file.index != crate::git::ChangeCode::Unchanged {
                    Color::Green
                } else {
                    Color::Yellow
                };
                let mut spans = vec![
                    Span::styled(
                        format!("{marker} "),
                        Style::default().fg(if selected {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }),
                    ),
                    Span::styled(
                        format!("{: <20}", file.indicator()),
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(file.path.to_string_lossy().into_owned()),
                ];
                if let Some(original) = &file.original_path {
                    spans.push(Span::styled(
                        format!("  <- {}", original.display()),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();
        let mut state = ListState::default();
        state.select(Some(self.focused));
        let list = List::new(items)
            .block(panel(" changed files ", Color::Cyan))
            .highlight_style(
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn draw_preview(&mut self, frame: &mut Frame, area: Rect) {
        let Some(file) = self.focused_file() else {
            frame.render_widget(
                Paragraph::new("Select a file to preview its diff.")
                    .block(panel(" lazy diff preview ", Color::DarkGray))
                    .centered(),
                area,
            );
            return;
        };
        let content = match self.diffs.get(&file.path) {
            None => {
                "Focused file has not been read yet.\n\nPress Enter to load its diff on demand."
                    .to_string()
            }
            Some(Ok(diff)) => diff.text.clone(),
            Some(Err(error)) => format!("Could not load diff:\n\n{error}"),
        };
        let title = if self.diffs.contains_key(&file.path) {
            format!(" diff: {} ", file.path.display())
        } else {
            format!(" lazy diff: {} ", file.path.display())
        };
        frame.render_widget(
            Paragraph::new(content)
                .block(panel(&title, Color::Cyan))
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn draw_diff(&mut self, frame: &mut Frame, area: Rect) {
        self.load_focused_diff();
        let Some(file) = self.focused_file() else {
            self.screen = Screen::Selection;
            return;
        };
        let content = match self.diffs.get(&file.path) {
            Some(Ok(diff)) => diff.text.clone(),
            Some(Err(error)) => format!("Could not load diff:\n\n{error}"),
            None => "Loading diff...".to_string(),
        };
        let title = format!(" diff: {}  Esc/b back ", file.path.display());
        frame.render_widget(
            Paragraph::new(content)
                .block(panel(&title, Color::Cyan))
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        let text = "FGIT file selection\n\n↑/↓ or j/k  move focused file\nSpace         select or unselect focused file\na             select all safe visible changes\nu             select tracked changes only\nn             clear selection\nEnter         load and open focused file diff\nh or ?        open this help\nEsc or b      return to the previous screen\nq             quit from the root screen\n\nMouse: click a row to focus it; click its checkbox to toggle it.\n\nThis phase never stages files and never creates a commit.";
        frame.render_widget(Clear, centered(area, 72, 18));
        frame.render_widget(
            Paragraph::new(text)
                .block(panel(" help  Esc / Enter / b close ", Color::Cyan))
                .wrap(Wrap { trim: true }),
            centered(area, 72, 18),
        );
    }

    fn draw_error(&self, frame: &mut Frame, area: Rect, message: &str) {
        let text = format!("{message}\n\nr retries the Git read. Esc or q returns to the shell.");
        frame.render_widget(
            Paragraph::new(text)
                .block(panel(" FGIT cannot continue ", Color::Red))
                .wrap(Wrap { trim: true }),
            centered(area, 76, 12),
        );
    }
}

fn layout_mode(area: Rect, file_count: usize) -> LayoutMode {
    if area.width < 52 || area.height < 14 {
        LayoutMode::TooSmall
    } else if file_count <= 6 && area.width >= 92 && area.height >= 23 {
        LayoutMode::Small
    } else {
        LayoutMode::Big
    }
}

fn panel<'a>(title: &'a str, color: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title)
}
fn centered(area: Rect, width_percent: u16, height: u16) -> Rect {
    let width = area
        .width
        .saturating_mul(width_percent)
        .saturating_div(100)
        .max(1);
    let height = height.min(area.height).max(1);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}
fn contains(area: Rect, point: (u16, u16)) -> bool {
    point.0 >= area.x
        && point.0 < area.x + area.width
        && point.1 >= area.y
        && point.1 < area.y + area.height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventState, MouseEvent};
    use ratatui::{Terminal, backend::TestBackend};

    fn app_with_files() -> App {
        App {
            screen: Screen::Selection,
            repository: Some(RepositoryState {
                root: PathBuf::from("/repo"),
                branch: "main".to_string(),
                files: vec![
                    FileChange {
                        path: PathBuf::from("changed.rs"),
                        original_path: None,
                        index: crate::git::ChangeCode::Modified,
                        worktree: crate::git::ChangeCode::Modified,
                    },
                    FileChange {
                        path: PathBuf::from("new.rs"),
                        original_path: None,
                        index: crate::git::ChangeCode::Untracked,
                        worktree: crate::git::ChangeCode::Untracked,
                    },
                ],
            }),
            repository_path: Some(PathBuf::from("/repo")),
            selected: HashSet::new(),
            focused: 0,
            diffs: HashMap::new(),
            hit_areas: HitAreas::default(),
        }
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn uses_compact_layout_only_when_files_and_terminal_fit() {
        assert_eq!(layout_mode(Rect::new(0, 0, 100, 30), 3), LayoutMode::Small);
        assert_eq!(layout_mode(Rect::new(0, 0, 100, 30), 7), LayoutMode::Big);
        assert_eq!(
            layout_mode(Rect::new(0, 0, 40, 30), 3),
            LayoutMode::TooSmall
        );
    }

    #[test]
    fn keyboard_mouse_and_resize_paths_are_safe() {
        let mut app = app_with_files();
        assert!(!app.handle_event(key(KeyCode::Char('j'))));
        assert_eq!(app.focused, 1);
        assert!(!app.handle_event(key(KeyCode::Char('k'))));
        assert_eq!(app.focused, 0);
        app.handle_event(key(KeyCode::Char(' ')));
        assert!(app.selected.contains(Path::new("changed.rs")));
        app.handle_event(key(KeyCode::Char('a')));
        assert_eq!(app.selected.len(), 2);
        app.handle_event(key(KeyCode::Char('u')));
        assert_eq!(app.selected.len(), 1);
        app.handle_event(key(KeyCode::Char('n')));
        assert!(app.selected.is_empty());
        app.handle_event(key(KeyCode::Char('h')));
        assert!(matches!(app.screen, Screen::Help));
        app.handle_event(key(KeyCode::Esc));
        assert!(matches!(app.screen, Screen::Selection));
        app.handle_event(key(KeyCode::Char('?')));
        assert!(matches!(app.screen, Screen::Help));
        app.handle_event(key(KeyCode::Char('b')));
        assert!(matches!(app.screen, Screen::Selection));

        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: app.hit_areas.files.x + 1,
            row: app.hit_areas.files.y + 1,
            modifiers: KeyModifiers::NONE,
        };
        app.handle_event(Event::Mouse(mouse));
        assert!(app.selected.contains(Path::new("changed.rs")));
        terminal.resize(Rect::new(0, 0, 40, 10)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();

        app.handle_event(key(KeyCode::Enter));
        assert!(matches!(app.screen, Screen::Diff));
        app.handle_event(key(KeyCode::Char('b')));
        assert!(matches!(app.screen, Screen::Selection));
        assert!(app.handle_event(key(KeyCode::Esc)));
        assert!(app.handle_event(key(KeyCode::Char('q'))));
    }
}
