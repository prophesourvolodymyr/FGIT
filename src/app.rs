use crate::{
    command::LaunchOptions,
    draft::{CommitDraft, DraftField, DraftMode},
    git::{DiffPreview, FileChange, RepositoryState},
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    Loading,
    Selection,
    Diff,
    Draft,
    Destination,
    Confirmation,
    Progress,
    Result,
    Help,
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayoutMode {
    Small,
    Big,
    TooSmall,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Destination {
    LocalOnly,
    Upstream,
    Remote(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionFocus {
    Files,
    Message,
    Destination,
}

#[derive(Default)]
struct HitAreas {
    files: Rect,
    message: Rect,
    destination: Rect,
    primary: Rect,
    back: Rect,
}

pub struct App {
    pub screen: Screen,
    pub repository: Option<RepositoryState>,
    repository_path: Option<PathBuf>,
    selected: HashSet<PathBuf>,
    focused: usize,
    file_columns: usize,
    file_scroll_row: usize,
    selection_focus: SelectionFocus,
    files_active: bool,
    message_editing: bool,
    launch: LaunchOptions,
    draft: CommitDraft,
    draft_field: usize,
    destination: Destination,
    destination_cursor: usize,
    diff: Option<DiffPreview>,
    result_hash: Option<String>,
    result_detail: String,
    progress: String,
    commit_pending: bool,
    progress_ticks: u8,
    animation_tick: usize,
    help_return: Screen,
    hit_areas: HitAreas,
}

impl App {
    pub fn loading_with_options(launch: LaunchOptions) -> Self {
        let draft = launch
            .message
            .clone()
            .map(CommitDraft::with_message)
            .unwrap_or_default();
        Self {
            screen: Screen::Loading,
            repository: None,
            repository_path: None,
            selected: HashSet::new(),
            focused: 0,
            file_columns: 1,
            file_scroll_row: 0,
            selection_focus: SelectionFocus::Files,
            files_active: false,
            message_editing: false,
            launch,
            draft,
            draft_field: 0,
            destination: Destination::LocalOnly,
            destination_cursor: 0,
            diff: None,
            result_hash: None,
            result_detail: String::new(),
            progress: String::new(),
            commit_pending: false,
            progress_ticks: 0,
            animation_tick: 0,
            help_return: Screen::Selection,
            hit_areas: HitAreas::default(),
        }
    }

    pub fn load_repository(&mut self, path: &Path) {
        self.repository_path = Some(path.to_path_buf());
        match RepositoryState::load(path) {
            Ok(repository) => {
                self.repository = Some(repository);
                self.select_all();
                self.screen = if self.launch.automatic && !self.launch.select_files {
                    if self.launch.choose_destination {
                        Screen::Destination
                    } else {
                        Screen::Confirmation
                    }
                } else {
                    Screen::Selection
                };
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
        match self.screen.clone() {
            Screen::Loading => self.draw_loading(frame, area),
            Screen::Selection => self.draw_selection(frame, area),
            Screen::Diff => self.draw_diff(frame, area),
            Screen::Draft => self.draw_draft(frame, area),
            Screen::Destination => self.draw_destination(frame, area),
            Screen::Confirmation => self.draw_confirmation(frame, area),
            Screen::Progress => self.draw_progress(frame, area),
            Screen::Result => self.draw_result(frame, area),
            Screen::Help => self.draw_help(frame, area),
            Screen::Error(message) => self.draw_error(frame, area, &message),
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
                    self.screen = self.help_return.clone();
                }
                false
            }
            Screen::Selection => self.selection_key(key),
            Screen::Diff => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('b') | KeyCode::Enter) {
                    self.screen = Screen::Selection;
                }
                false
            }
            Screen::Draft => self.draft_key(key),
            Screen::Destination => self.destination_key(key),
            Screen::Confirmation => self.confirmation_key(key),
            Screen::Progress => false,
            Screen::Result => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    true
                } else if matches!(key.code, KeyCode::Enter | KeyCode::Char('n')) {
                    self.reload_selection();
                    false
                } else {
                    false
                }
            }
        }
    }

    pub fn process_pending(&mut self) {
        if self.commit_pending {
            if self.progress_ticks < 3 {
                self.progress_ticks += 1;
                return;
            }
            self.commit_pending = false;
            self.run_commit();
        }
    }

    pub fn tick_animation(&mut self) {
        self.animation_tick = self.animation_tick.wrapping_add(1);
    }

    fn selection_key(&mut self, key: KeyEvent) -> bool {
        if self.message_editing {
            return self.inline_message_key(key);
        }
        if self.files_active {
            return self.active_files_key(key);
        }
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            return true;
        }
        match key.code {
            KeyCode::Char('h') | KeyCode::Char('?') => {
                self.open_help(Screen::Selection);
            }
            KeyCode::Left => self.selection_focus = SelectionFocus::Files,
            KeyCode::Right => {
                self.selection_focus = match self.selection_focus {
                    SelectionFocus::Files => SelectionFocus::Message,
                    SelectionFocus::Message => SelectionFocus::Destination,
                    SelectionFocus::Destination => SelectionFocus::Destination,
                }
            }
            KeyCode::Up | KeyCode::Char('k') => match self.selection_focus {
                SelectionFocus::Files => self.focused = self.focused.saturating_sub(1),
                SelectionFocus::Message => self.selection_focus = SelectionFocus::Files,
                SelectionFocus::Destination => self.selection_focus = SelectionFocus::Message,
            },
            KeyCode::Down | KeyCode::Char('j') => match self.selection_focus {
                SelectionFocus::Files => {
                    self.focused = (self.focused + 1).min(self.file_count().saturating_sub(1))
                }
                SelectionFocus::Message => self.selection_focus = SelectionFocus::Destination,
                SelectionFocus::Destination => self.selection_focus = SelectionFocus::Destination,
            },
            KeyCode::Char(' ') if self.selection_focus == SelectionFocus::Files => {
                self.files_active = true;
                self.toggle_focused();
            }
            KeyCode::Char(' ') if self.selection_focus == SelectionFocus::Message => {
                self.open_message_editor();
            }
            KeyCode::Char(' ') if self.selection_focus == SelectionFocus::Destination => {
                self.screen = Screen::Destination;
            }
            KeyCode::Char('a') => self.select_all(),
            KeyCode::Char('n') => self.selected.clear(),
            KeyCode::Char('d') if self.selection_focus == SelectionFocus::Files => self.open_diff(),
            KeyCode::Enter => match self.selection_focus {
                SelectionFocus::Files => self.files_active = true,
                SelectionFocus::Message => self.open_message_editor(),
                SelectionFocus::Destination => self.screen = Screen::Destination,
            },
            KeyCode::Char('c') => self.open_message_editor(),
            KeyCode::Char('r') => self.screen = Screen::Destination,
            _ => {}
        }
        false
    }

    fn active_files_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('q') {
            return true;
        }
        match key.code {
            KeyCode::Esc => self.files_active = false,
            KeyCode::Enter => {
                self.files_active = false;
                self.selection_focus = SelectionFocus::Message;
            }
            KeyCode::Char(' ') => self.toggle_focused(),
            KeyCode::Left if !self.focused.is_multiple_of(self.file_columns) => self.focused -= 1,
            KeyCode::Right
                if self.focused + 1 < self.file_count()
                    && self.focused % self.file_columns < self.file_columns - 1 =>
            {
                self.focused += 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.focused = self.focused.saturating_sub(self.file_columns);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.focused =
                    (self.focused + self.file_columns).min(self.file_count().saturating_sub(1));
            }
            KeyCode::Char('d') => self.open_diff(),
            KeyCode::Char('c') => self.open_message_editor(),
            KeyCode::Char('r') => {
                self.files_active = false;
                self.screen = Screen::Destination;
            }
            KeyCode::Char('h') | KeyCode::Char('?') => self.open_help(Screen::Selection),
            _ => {}
        }
        false
    }

    fn inline_message_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('b') => self.message_editing = false,
            KeyCode::Enter if self.draft.validation_error().is_none() => {
                self.message_editing = false;
                self.selection_focus = SelectionFocus::Destination;
            }
            KeyCode::Backspace => {
                self.draft.summary.pop();
            }
            KeyCode::Char(character) => self.draft.summary.push(character),
            _ => {}
        }
        false
    }

    fn draft_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('a') {
            self.draft.mode = if self.draft.mode == DraftMode::Auto {
                DraftMode::Structured
            } else {
                DraftMode::Auto
            };
            return false;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('b') => self.screen = Screen::Selection,
            KeyCode::Char('q') => return true,
            KeyCode::Char('h') | KeyCode::Char('?') => self.open_help(Screen::Draft),
            KeyCode::Char(' ') if self.draft.mode == DraftMode::Auto => {
                self.draft.mode = DraftMode::Structured;
                self.draft_field = DraftField::ALL
                    .iter()
                    .position(|field| *field == DraftField::Summary)
                    .unwrap_or(0);
            }
            KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                self.draft_field = (self.draft_field + 1) % DraftField::ALL.len()
            }
            KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                self.draft_field =
                    (self.draft_field + DraftField::ALL.len() - 1) % DraftField::ALL.len()
            }
            KeyCode::Char(' ') if DraftField::ALL[self.draft_field] == DraftField::Breaking => {
                self.draft.breaking = !self.draft.breaking
            }
            KeyCode::Backspace => {
                if let Some(value) = self.draft.field_mut(DraftField::ALL[self.draft_field]) {
                    value.pop();
                }
            }
            KeyCode::Enter if self.draft.validation_error().is_none() => {
                self.screen = Screen::Destination
            }
            KeyCode::Char(character)
                if matches!(self.draft.mode, DraftMode::Structured | DraftMode::Custom) =>
            {
                if let Some(value) = self.draft.field_mut(DraftField::ALL[self.draft_field]) {
                    value.push(character);
                }
            }
            _ => {}
        }
        false
    }

    fn destination_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('b') => self.screen = Screen::Selection,
            KeyCode::Char('q') => return true,
            KeyCode::Char('h') | KeyCode::Char('?') => self.open_help(Screen::Destination),
            KeyCode::Up | KeyCode::Char('k') => {
                self.destination_cursor = self.destination_cursor.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.destination_cursor =
                    (self.destination_cursor + 1).min(self.destinations().len().saturating_sub(1))
            }
            KeyCode::Char(' ') => self.choose_destination(),
            KeyCode::Enter => {
                self.choose_destination();
                self.screen = Screen::Confirmation;
            }
            _ => {}
        }
        false
    }

    fn confirmation_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('b') => self.screen = Screen::Destination,
            KeyCode::Char('q') => return true,
            KeyCode::Char('h') | KeyCode::Char('?') => self.open_help(Screen::Confirmation),
            KeyCode::Enter | KeyCode::Char(' ') => self.begin_commit(),
            _ => {}
        }
        false
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return;
        }
        let point = (mouse.column, mouse.row);
        if self.screen == Screen::Help {
            self.screen = self.help_return.clone();
            return;
        }
        if self.screen == Screen::Selection {
            if contains(self.hit_areas.message, point) {
                self.selection_focus = SelectionFocus::Message;
                self.open_message_editor();
            } else if contains(self.hit_areas.destination, point) {
                self.selection_focus = SelectionFocus::Destination;
                self.screen = Screen::Destination;
            } else if contains(self.hit_areas.files, point) {
                let inner = panel("", Color::Reset).inner(self.hit_areas.files);
                let columns = file_grid_columns(inner.width);
                let visible_rows = (inner.height / 4).max(1) as usize;
                let row_height = (inner.height / visible_rows as u16).max(4);
                let column = (mouse.column.saturating_sub(inner.x) as usize * columns
                    / inner.width.max(1) as usize)
                    .min(columns.saturating_sub(1));
                let row = mouse.row.saturating_sub(inner.y) as usize / row_height as usize;
                let index = (self.file_scroll_row + row) * columns + column;
                if index < self.file_count() {
                    self.selection_focus = SelectionFocus::Files;
                    self.files_active = true;
                    self.focused = index;
                }
            }
        } else if self.screen == Screen::Confirmation && contains(self.hit_areas.primary, point) {
            self.begin_commit();
        } else if contains(self.hit_areas.back, point) {
            self.screen = Screen::Selection;
        }
    }

    fn begin_commit(&mut self) {
        let Some(repository) = self.repository.clone() else {
            return;
        };
        if repository.files.iter().any(FileChange::is_conflict) {
            self.screen = Screen::Error("Resolve conflicts before committing.".to_string());
            return;
        }
        self.screen = Screen::Progress;
        self.progress = if self.destination == Destination::LocalOnly {
            "STAGING / COMMITTING".to_string()
        } else {
            "STAGING / COMMIT / PUSHING".to_string()
        };
        self.progress_ticks = 0;
        self.commit_pending = true;
    }

    fn run_commit(&mut self) {
        let Some(repository) = self.repository.clone() else {
            return;
        };
        let mut paths: Vec<_> = self.selected.iter().cloned().collect();
        paths.sort();
        match repository.commit_selected(&paths, &self.draft.message()) {
            Ok(result) => {
                self.result_hash = Some(result.short_hash);
                self.result_detail = "LOCAL COMMIT OK".to_string();
                let push_result = match self.destination.clone() {
                    Destination::LocalOnly => Ok(None),
                    Destination::Upstream => repository
                        .upstream
                        .as_ref()
                        .map(|upstream| {
                            repository
                                .push(&upstream.remote, &upstream.branch)
                                .map(Some)
                        })
                        .unwrap_or(Ok(None)),
                    Destination::Remote(remote) => {
                        repository.push(&remote, &repository.branch).map(Some)
                    }
                };
                match push_result {
                    Ok(Some(_)) => self.result_detail.push_str("\nPUSH OK"),
                    Ok(None) => {}
                    Err(_) => self.result_detail.push_str("\nPUSH FAILED"),
                }
                self.screen = Screen::Result;
            }
            Err(error) => self.screen = Screen::Error(error.to_string()),
        }
    }

    fn reload_selection(&mut self) {
        if let Some(path) = self.repository_path.clone() {
            self.selected.clear();
            self.focused = 0;
            self.file_scroll_row = 0;
            self.load_repository(&path);
        }
    }
    fn open_help(&mut self, return_to: Screen) {
        self.help_return = return_to;
        self.screen = Screen::Help;
    }
    fn open_message_editor(&mut self) {
        self.selection_focus = SelectionFocus::Message;
        self.files_active = false;
        if self.draft.mode == DraftMode::Auto {
            self.draft.mode = DraftMode::Structured;
        }
        self.draft_field = DraftField::ALL
            .iter()
            .position(|field| *field == DraftField::Summary)
            .unwrap_or(0);
        self.message_editing = true;
        self.screen = Screen::Selection;
    }
    fn file_count(&self) -> usize {
        self.repository
            .as_ref()
            .map_or(0, |repository| repository.files.len())
    }
    fn select_all(&mut self) {
        if let Some(repository) = &self.repository {
            self.selected = repository
                .files
                .iter()
                .filter(|file| file.is_safe_to_select())
                .map(|file| file.path.clone())
                .collect();
        }
    }
    fn toggle_focused(&mut self) {
        let Some(file) = self
            .repository
            .as_ref()
            .and_then(|repository| repository.files.get(self.focused))
        else {
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
    fn open_diff(&mut self) {
        if let Some(file) = self
            .repository
            .as_ref()
            .and_then(|repository| repository.files.get(self.focused))
        {
            self.diff = self
                .repository
                .as_ref()
                .and_then(|repository| repository.diff_for(file).ok());
            self.screen = Screen::Diff;
        }
    }

    fn destinations(&self) -> Vec<(String, Destination)> {
        let mut options = vec![("Local only".to_string(), Destination::LocalOnly)];
        if let Some(repository) = &self.repository {
            if repository.upstream.is_some() {
                options.push(("Configured upstream".to_string(), Destination::Upstream));
            }
            options.extend(
                repository
                    .remotes
                    .iter()
                    .cloned()
                    .map(|remote| (format!("Push to {remote}"), Destination::Remote(remote))),
            );
        }
        options
    }
    fn choose_destination(&mut self) {
        if let Some((_, destination)) = self.destinations().get(self.destination_cursor).cloned() {
            self.destination = destination;
        }
    }
    fn destination_choices_text(&self) -> String {
        self.destinations()
            .into_iter()
            .map(|(label, destination)| {
                let marker = if self.destination == destination {
                    "*"
                } else {
                    " "
                };
                format!("{marker} {}", label.to_uppercase())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn command_preview(&self) -> String {
        let mut paths: Vec<_> = self
            .selected
            .iter()
            .map(|path| display_quote(&path.to_string_lossy()))
            .collect();
        paths.sort();
        let add = if paths.is_empty() {
            "git add -- <none>".to_string()
        } else {
            format!("git add -- {}", paths.join(" "))
        };
        let commit = format!("git commit -m {}", display_quote(&self.draft.message()));
        let push = match &self.destination {
            Destination::LocalOnly => "local only".to_string(),
            Destination::Upstream => self
                .repository
                .as_ref()
                .and_then(|repository| repository.upstream.as_ref())
                .map(|upstream| format!("git push {} {}", upstream.remote, upstream.branch))
                .unwrap_or_else(|| "local only".to_string()),
            Destination::Remote(remote) => self
                .repository
                .as_ref()
                .map(|repository| format!("git push {} {}", remote, repository.branch))
                .unwrap_or_else(|| format!("git push {remote}")),
        };
        format!("{add}\n{commit}\n{push}")
    }

    fn draw_loading(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new("FGIT\n\nReading repository state...")
                .block(panel(" loading ", Color::Cyan))
                .alignment(Alignment::Center),
            centered(area, 58, 9),
        );
    }
    fn draw_too_small(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new(format!("FGIT needs at least 52 columns x 14 rows.\nCurrent terminal: {} x {}.\n\nResize, then FGIT will redraw safely.\n\nq or Esc quits.", area.width, area.height)).block(panel(" terminal too small ", Color::Yellow)).wrap(Wrap { trim: true }), area);
    }

    fn draw_selection(&mut self, frame: &mut Frame, area: Rect) {
        let Some(repository) = &self.repository else {
            return;
        };
        let title = format!(
            " FGIT  {}  {} changed  {} selected ",
            repository.branch,
            repository.files.len(),
            self.selected.len()
        );
        let outer = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);
        let inner = outer.inner(area);
        frame.render_widget(outer, area);
        let footer_height = 2;
        let body_area = Rect::new(
            inner.x,
            inner.y,
            inner.width,
            inner.height.saturating_sub(footer_height),
        );
        let footer_area = Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(footer_height),
            inner.width,
            footer_height.min(inner.height),
        );
        let body = match layout_mode(body_area, repository.files.len()) {
            LayoutMode::Small => Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(42),
                    Constraint::Percentage(33),
                    Constraint::Percentage(25),
                ])
                .split(body_area),
            LayoutMode::Big => {
                let stack = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(54),
                        Constraint::Percentage(27),
                        Constraint::Percentage(19),
                    ])
                    .split(body_area);
                self.draw_files(frame, stack[0]);
                self.draw_message(frame, stack[1]);
                self.draw_destination_summary(frame, stack[2]);
                draw_vertical_flow_arrow(frame, stack[0], stack[1]);
                draw_vertical_flow_arrow(frame, stack[1], stack[2]);
                self.draw_footer(frame, footer_area);
                return;
            }
            LayoutMode::TooSmall => {
                self.draw_too_small(frame, area);
                return;
            }
        };
        self.draw_files(frame, body[0]);
        self.draw_message(frame, body[1]);
        self.draw_destination_summary(frame, body[2]);
        draw_horizontal_flow_arrow(frame, body[0], body[1]);
        draw_horizontal_flow_arrow(frame, body[1], body[2]);
        self.draw_footer(frame, footer_area);
    }

    fn draw_files(&mut self, frame: &mut Frame, area: Rect) {
        self.hit_areas.files = area;
        let Some(repository) = &self.repository else {
            return;
        };
        if repository.files.is_empty() {
            frame.render_widget(
                Paragraph::new("Nothing to commit.\n\nWorking tree is clean.")
                    .block(panel(" GIT ADD ", Color::Green))
                    .alignment(Alignment::Center),
                area,
            );
            return;
        }
        let outer = panel(
            " GIT ADD ",
            if self.files_active {
                Color::White
            } else if self.selection_focus == SelectionFocus::Files {
                Color::Cyan
            } else {
                Color::DarkGray
            },
        );
        let inner = outer.inner(area);
        frame.render_widget(outer, area);
        if self.selection_focus == SelectionFocus::Files && !self.files_active {
            draw_border_marker(frame, area, self.animation_tick, Color::Cyan);
        }
        let columns = file_grid_columns(inner.width);
        self.file_columns = columns;
        let total_rows = repository.files.len().div_ceil(columns);
        let visible_rows = (inner.height / 4).max(1) as usize;
        let focus_row = self.focused / columns;
        let max_scroll = total_rows.saturating_sub(visible_rows);
        if focus_row < self.file_scroll_row {
            self.file_scroll_row = focus_row;
        } else if focus_row >= self.file_scroll_row + visible_rows {
            self.file_scroll_row = focus_row + 1 - visible_rows;
        }
        self.file_scroll_row = self.file_scroll_row.min(max_scroll);
        let row_height = (inner.height / visible_rows as u16).max(4);
        let start = self.file_scroll_row * columns;
        let end = ((self.file_scroll_row + visible_rows) * columns).min(repository.files.len());
        for index in start..end {
            let file = &repository.files[index];
            let column = index % columns;
            let row = index / columns - self.file_scroll_row;
            let cell_x = inner.x + inner.width * column as u16 / columns as u16;
            let next_cell_x = inner.x + inner.width * (column + 1) as u16 / columns as u16;
            let cell_y = inner.y + row as u16 * row_height;
            let x = cell_x.saturating_add(1);
            let y = cell_y.saturating_add(1);
            let card = Rect::new(
                x,
                y,
                next_cell_x.saturating_sub(cell_x).saturating_sub(1),
                row_height
                    .saturating_sub(if row_height >= 5 { 1 } else { 0 })
                    .min(inner.y + inner.height.saturating_sub(y)),
            );
            let selected = self.selected.contains(&file.path);
            let focused = self.selection_focus == SelectionFocus::Files && self.focused == index;
            let marker = if selected { "[x]" } else { "[ ]" };
            let card_style = if selected {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            let content = vec![
                Line::styled(
                    format!("{marker} {}", compact_path(&file.path)),
                    card_style.add_modifier(Modifier::BOLD),
                ),
                if file.is_conflict() {
                    Line::styled("conflict", card_style.fg(Color::LightRed))
                } else {
                    Line::from(vec![
                        Span::styled(format!("+{}", file.additions), card_style.fg(Color::Green)),
                        Span::raw("  "),
                        Span::styled(
                            format!("-{}", file.deletions),
                            card_style.fg(Color::LightRed),
                        ),
                    ])
                },
            ];
            let border = if selected {
                Color::White
            } else if focused {
                Color::Cyan
            } else if file.is_conflict() {
                Color::LightRed
            } else {
                Color::DarkGray
            };
            frame.render_widget(
                Paragraph::new(content)
                    .style(card_style)
                    .block(panel("", border).style(card_style))
                    .wrap(Wrap { trim: true }),
                card,
            );
            if focused && !selected {
                draw_border_marker(frame, card, self.animation_tick, Color::Cyan);
            }
        }
    }

    fn draw_message(&mut self, frame: &mut Frame, area: Rect) {
        self.hit_areas.message = area;
        let message_body = if self.message_editing {
            format!("> {}|", self.draft.summary)
        } else if self.draft.mode == DraftMode::Auto {
            "AUTO COMMIT".to_string()
        } else {
            self.draft.message()
        };
        let border = if self.message_editing {
            Color::White
        } else if self.selection_focus == SelectionFocus::Message {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        frame.render_widget(
            Paragraph::new(message_body)
                .block(panel(" GIT COMMIT ", border))
                .wrap(Wrap { trim: false }),
            area,
        );
        if self.selection_focus == SelectionFocus::Message && !self.message_editing {
            draw_border_marker(frame, area, self.animation_tick, Color::Cyan);
        }
    }
    fn draw_destination_summary(&mut self, frame: &mut Frame, area: Rect) {
        self.hit_areas.destination = area;
        frame.render_widget(
            Paragraph::new(self.destination_choices_text())
                .block(panel(
                    " GIT PUSH ",
                    if self.selection_focus == SelectionFocus::Destination {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    },
                ))
                .wrap(Wrap { trim: true }),
            area,
        );
        if self.selection_focus == SelectionFocus::Destination {
            draw_border_marker(frame, area, self.animation_tick, Color::Yellow);
        }
    }
    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(
                "↑↓ move  ←→ stages  Space select/open  Enter next  d diff  Esc back  / commands",
            )
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn draw_diff(&self, frame: &mut Frame, area: Rect) {
        let text = self
            .diff
            .as_ref()
            .map(|diff| diff.text.clone())
            .unwrap_or_else(|| "No textual diff is available.".to_string());
        frame.render_widget(
            Paragraph::new(text)
                .block(panel(" diff  Enter / b back ", Color::Cyan))
                .wrap(Wrap { trim: false }),
            area,
        );
    }
    fn draw_draft(&self, frame: &mut Frame, area: Rect) {
        let lines: Vec<_> = DraftField::ALL
            .into_iter()
            .map(|field| {
                let focused = DraftField::ALL[self.draft_field] == field;
                let value = if field == DraftField::Breaking {
                    if self.draft.breaking { "yes" } else { "no" }
                } else {
                    self.draft.field(field)
                };
                Line::styled(
                    format!(
                        "{} {: <9} {value}",
                        if focused { ">" } else { " " },
                        field.label()
                    ),
                    Style::default().fg(if focused { Color::Cyan } else { Color::Reset }),
                )
            })
            .collect();
        let detail = format!(
            "{}\n\nTab/arrows move fields  Ctrl+a changes auto/structured\nEnter destination  b back",
            self.draft.message()
        );
        let parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(centered(area, 82, area.height.saturating_sub(4)));
        frame.render_widget(
            Paragraph::new(lines).block(panel(" GIT COMMIT ", Color::Cyan)),
            parts[0],
        );
        frame.render_widget(
            Paragraph::new(detail)
                .block(panel(" message preview ", Color::Green))
                .wrap(Wrap { trim: false }),
            parts[1],
        );
    }
    fn draw_destination(&self, frame: &mut Frame, area: Rect) {
        let options = self.destinations();
        let lines: Vec<_> = options
            .iter()
            .enumerate()
            .map(|(index, (label, _))| {
                Line::styled(
                    format!(
                        "{} {}",
                        if self.destination_cursor == index {
                            ">"
                        } else {
                            " "
                        },
                        label.to_uppercase()
                    ),
                    Style::default()
                        .fg(if self.destination_cursor == index {
                            Color::Cyan
                        } else {
                            Color::Reset
                        })
                        .add_modifier(if self.destination_cursor == index {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                )
            })
            .collect();
        frame.render_widget(
            Paragraph::new(lines)
                .block(panel(" GIT PUSH ", Color::Yellow))
                .wrap(Wrap { trim: true }),
            centered(area, 78, area.height.saturating_sub(6)),
        );
    }
    fn draw_confirmation(&mut self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "F U C K I N G\n\nP U S H\n\nREADY\n{} FILES\n\n{}",
            self.selected.len(),
            self.command_preview()
        );
        let text_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(6));
        let button_y = area.y + area.height.saturating_sub(5);
        let button_width = 20;
        let gap = 4;
        let total_width = button_width * 2 + gap;
        let button_x = area.x + area.width.saturating_sub(total_width) / 2;
        let commit_button = Rect::new(button_x, button_y, button_width, 3);
        let back_button = Rect::new(button_x + button_width + gap, button_y, button_width, 3);
        self.hit_areas.primary = commit_button;
        self.hit_areas.back = back_button;
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(full_screen_text(text_area, text))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow)),
            text_area,
        );
        frame.render_widget(
            Paragraph::new("ENTER COMMIT")
                .block(panel("", Color::Yellow))
                .alignment(Alignment::Center),
            commit_button,
        );
        frame.render_widget(
            Paragraph::new("ESC BACK")
                .block(panel("", Color::DarkGray))
                .alignment(Alignment::Center),
            back_button,
        );
    }
    fn draw_progress(&self, frame: &mut Frame, area: Rect) {
        let width = area.width.saturating_sub(20).clamp(12, 72) as usize;
        let filled = width * self.progress_ticks as usize / 3;
        let bar = format!(
            "[{}{}]",
            "=".repeat(filled),
            ".".repeat(width.saturating_sub(filled))
        );
        let text = format!(
            "F U C K I N G  P U S H\n\n{}\n\n{}\n{} FILES",
            self.progress,
            bar,
            self.selected.len()
        );
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(full_screen_text(area, text))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Cyan)),
            area,
        );
    }
    fn draw_result(&self, frame: &mut Frame, area: Rect) {
        let hash = self.result_hash.as_deref().unwrap_or("complete");
        let text = format!(
            "C O M M I T\n\n{hash}\n\n{}\n\nENTER AGAIN   Q QUIT",
            self.result_detail
        );
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(full_screen_text(area, text))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Green)),
            area,
        );
    }
    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        let text = "FGIT\n\nSelection: arrows/j/k move; Space toggles; a selects all; n clears; Enter opens a diff.\n\nClick the message or destination panels, or press c/r. Enter moves forward. Escape and b return.\n\nThe local commit happens before any optional push.\n\nEsc, Enter, b, h, or ? closes this help.";
        let box_area = centered(area, 74, 16);
        frame.render_widget(Clear, box_area);
        frame.render_widget(
            Paragraph::new(text)
                .block(panel(" help ", Color::Cyan))
                .wrap(Wrap { trim: true }),
            box_area,
        );
    }
    fn draw_error(&self, frame: &mut Frame, area: Rect, message: &str) {
        frame.render_widget(
            Paragraph::new(format!(
                "{message}\n\nr retries repository loading. Esc or q returns to the shell."
            ))
            .block(panel(" FGIT cannot continue ", Color::Red))
            .wrap(Wrap { trim: true }),
            centered(area, 76, 12),
        );
    }
}

fn layout_mode(area: Rect, files: usize) -> LayoutMode {
    if area.width < 52 || area.height < 14 {
        LayoutMode::TooSmall
    } else if files <= 6 && area.width >= 94 && area.height >= 23 {
        LayoutMode::Small
    } else {
        LayoutMode::Big
    }
}

fn file_grid_columns(width: u16) -> usize {
    if width >= 72 {
        4
    } else if width >= 52 {
        3
    } else {
        2
    }
}

fn compact_path(path: &Path) -> String {
    let value = path
        .file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy();
    if value.chars().count() <= 18 {
        return value.into_owned();
    }
    let suffix: String = value.chars().skip(value.chars().count() - 15).collect();
    format!("...{suffix}")
}

fn full_screen_text(area: Rect, text: String) -> String {
    let lines = text.lines().count() as u16;
    let padding = "\n".repeat(area.height.saturating_sub(lines + 1) as usize / 2);
    format!("{padding}{text}")
}

fn display_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn draw_border_marker(frame: &mut Frame, area: Rect, tick: usize, color: Color) {
    if area.width < 3 || area.height < 3 {
        return;
    }
    let mut points = Vec::with_capacity((area.width + area.height) as usize * 2);
    for x in area.x + 1..area.x + area.width - 1 {
        points.push((x, area.y));
    }
    for y in area.y + 1..area.y + area.height - 1 {
        points.push((area.x + area.width - 1, y));
    }
    for x in (area.x + 1..area.x + area.width - 1).rev() {
        points.push((x, area.y + area.height - 1));
    }
    for y in (area.y + 1..area.y + area.height - 1).rev() {
        points.push((area.x, y));
    }
    let start = tick.wrapping_mul(2) % points.len();
    for offset in 0..4 {
        let (x, y) = points[(start + offset) % points.len()];
        let glyph = if y == area.y || y == area.y + area.height - 1 {
            "="
        } else {
            "|"
        };
        frame.render_widget(
            Paragraph::new(glyph).style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Rect::new(x, y, 1, 1),
        );
    }
}

fn draw_horizontal_flow_arrow(frame: &mut Frame, left: Rect, right: Rect) {
    let x = left.x + left.width.saturating_sub(1);
    let y = left.y + left.height / 2;
    frame.render_widget(
        Paragraph::new("->").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Rect::new(x, y, 2.min(right.x.saturating_sub(x).max(1)), 1),
    );
}

fn draw_vertical_flow_arrow(frame: &mut Frame, top: Rect, _bottom: Rect) {
    let x = top.x + top.width / 2;
    let y = top.y + top.height.saturating_sub(1);
    frame.render_widget(
        Paragraph::new("v").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Rect::new(x, y, 1, 1),
    );
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
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height.min(area.height)) / 2,
        width,
        height.min(area.height).max(1),
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
    use crate::git::{ChangeCode, RepositoryState};
    use crossterm::event::KeyEventState;
    use ratatui::{Terminal, backend::TestBackend};

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }
    fn app() -> App {
        let mut app = App::loading_with_options(LaunchOptions::default());
        app.screen = Screen::Selection;
        app.repository = Some(RepositoryState {
            root: PathBuf::from("/repo"),
            branch: "main".to_string(),
            upstream: None,
            remotes: vec![],
            files: vec![FileChange {
                path: PathBuf::from("one.rs"),
                original_path: None,
                index: ChangeCode::Modified,
                worktree: ChangeCode::Modified,
                additions: 67,
                deletions: 44,
            }],
        });
        app
    }
    #[test]
    fn blueprint_flow_has_small_and_big_layouts() {
        assert_eq!(layout_mode(Rect::new(0, 0, 100, 30), 3), LayoutMode::Small);
        assert_eq!(layout_mode(Rect::new(0, 0, 100, 30), 7), LayoutMode::Big);
    }
    #[test]
    fn selection_reaches_confirmation_without_writing() {
        let mut app = app();
        app.handle_event(key(KeyCode::Char(' ')));
        app.handle_event(key(KeyCode::Char('c')));
        assert_eq!(app.screen, Screen::Selection);
        assert!(app.message_editing);
        app.handle_event(key(KeyCode::Char('t')));
        app.handle_event(key(KeyCode::Char('e')));
        app.handle_event(key(KeyCode::Char('s')));
        app.handle_event(key(KeyCode::Char('t')));
        app.handle_event(key(KeyCode::Enter));
        app.handle_event(key(KeyCode::Char(' ')));
        app.handle_event(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Confirmation);
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
    }
}
