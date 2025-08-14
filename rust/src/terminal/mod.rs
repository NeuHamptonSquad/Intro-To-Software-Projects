use std::{cell::RefCell, io::stdout, ops::Deref, time::Duration};

use ansi_to_tui::IntoText;
use clap::Parser;
use godot::{
    classes::{InputEvent, InputEventKey},
    prelude::*,
};
use ratatui::{
    DefaultTerminal,
    crossterm::{
        self,
        event::{
            self, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MediaKeyCode,
            ModifierKeyCode,
        },
    },
    layout::{Alignment, Constraint, Direction, Layout, Offset, Rect},
    prelude::CrosstermBackend,
    text::Text,
    widgets::{Block, Borders, Paragraph},
};
use tachyonfx::{Effect, Interpolation, RefCount, Shader, fx};
use tracing::instrument;
use yoke::{Yoke, Yokeable};

use crate::terminal::{
    commands::{
        main::{MainCli, MainCommands},
        pause::{PauseCli, PauseCommands},
    },
    cursor_x::CursorX,
    tile_map::TerminalTileMap,
};

mod commands;
mod cursor_x;
mod tile_map;

#[derive(Yokeable)]
pub struct YokeableText<'a>(Text<'a>);

#[derive(Default, Clone, Copy)]
enum TerminalState {
    #[default]
    MainView,
}

#[derive(GodotClass)]
#[class(base=Node)]
/// This class is responsible for the terminal side
/// of this game.
pub struct Terminal {
    base: Base<Node>,
    main_tile_map: Option<Gd<TerminalTileMap>>,
    terminal: DefaultTerminal,
    terminal_command: String,
    terminal_command_output: Yoke<YokeableText<'static>, String>,
    terminal_state: RefCount<TerminalState>,
    cursor_x: CursorX,
    effect: Effect,
    initialized: bool,
    paused: bool,
}

#[godot_api]
impl INode for Terminal {
    #[instrument(skip_all)]
    fn init(base: Base<Node>) -> Self {
        Self {
            main_tile_map: None,
            base,
            terminal: ratatui::Terminal::new(CrosstermBackend::new(stdout())).unwrap(),
            terminal_command: String::new(),
            terminal_command_output: Yoke::attach_to_cart(
                String::from("Try typing `help` to get started"),
                |cart| YokeableText(Text::from(cart)),
            ),
            terminal_state: RefCount::new(RefCell::new(TerminalState::default())),
            cursor_x: CursorX::default(),
            effect: fx::coalesce((1000, Interpolation::Linear)),
            initialized: false,
            paused: false,
        }
    }

    #[instrument(skip_all)]
    fn ready(&mut self) {
        self.terminal = ratatui::init();
        self.main_tile_map = self.base().try_get_node_as("MainTileMap");
        self.signals().init().connect_self(Self::_on_init);
        self.signals().pause().connect_self(Self::_on_pause);
    }

    #[instrument(skip_all)]
    fn process(&mut self, delta: f64) {
        if crossterm::event::poll(Duration::from_secs(0)).unwrap_or_default() {
            self.event(crossterm::event::read().unwrap());
        }
        self.terminal
            .draw(|frame| {
                let area = frame.area();
                let [main_area, error_area, terminal_area]: [Rect; 3] = Layout::new(
                    Direction::Vertical,
                    [
                        Constraint::Min(0),
                        Constraint::Length(
                            self.terminal_command_output.get().0.height() as u16 + 2,
                        ),
                        Constraint::Length(3),
                    ],
                )
                .split(area)
                .as_ref()
                .try_into()
                .unwrap();
                frame.render_widget(
                    Paragraph::new(self.terminal_command.as_str())
                        .block(Block::new().borders(Borders::all())),
                    terminal_area,
                );
                frame.render_widget(
                    Paragraph::new(self.terminal_command_output.get().0.clone())
                        .block(Block::new().borders(Borders::all())),
                    error_area,
                );
                frame.set_cursor_position(terminal_area.offset(Offset {
                    x: self.cursor_x.column() as i32 + 1,
                    y: 1,
                }));
                if self.paused {
                    let paused_text = Text::from("Paused");
                    let center = Rect::new(
                        main_area.x,
                        main_area.y + (main_area.height / 2),
                        main_area.width,
                        paused_text.height() as u16,
                    );
                    frame.render_widget(
                        Paragraph::new(paused_text).alignment(Alignment::Center),
                        center,
                    );
                } else {
                    if self.initialized {
                        match *self.terminal_state.borrow() {
                            TerminalState::MainView => {
                                if let Some(main_tile_map) = &self.main_tile_map {
                                    let main_tile_map = main_tile_map.bind();
                                    let used_rect = main_tile_map.get_used_rect();
                                    let center = Layout::new(
                                        Direction::Horizontal,
                                        [Constraint::Length(used_rect.width)],
                                    )
                                    .flex(ratatui::layout::Flex::Center)
                                    .split(main_area)[0];
                                    frame.render_widget(main_tile_map.deref(), center);
                                }
                            }
                        }
                    }
                    self.effect.process(
                        Duration::from_secs_f64(delta),
                        frame.buffer_mut(),
                        main_area,
                    );
                }
            })
            .unwrap();
    }

    #[instrument(skip(self))]
    fn input(&mut self, input: Gd<InputEvent>) {
        if let Ok(input_event_key) = input.try_cast::<InputEventKey>() {
            let code = match input_event_key.get_keycode() {
                godot::global::Key::ESCAPE => KeyCode::Esc,
                godot::global::Key::TAB => KeyCode::Tab,
                godot::global::Key::BACKTAB => KeyCode::BackTab,
                godot::global::Key::BACKSPACE => KeyCode::Backspace,
                godot::global::Key::ENTER | godot::global::Key::KP_ENTER => KeyCode::Enter,
                godot::global::Key::INSERT => KeyCode::Insert,
                godot::global::Key::DELETE => KeyCode::Delete,
                godot::global::Key::PAUSE => KeyCode::Pause,
                godot::global::Key::PRINT | godot::global::Key::SYSREQ => KeyCode::PrintScreen,
                godot::global::Key::CLEAR => KeyCode::Null,
                godot::global::Key::HOME => KeyCode::Home,
                godot::global::Key::END => KeyCode::End,
                godot::global::Key::LEFT => KeyCode::Left,
                godot::global::Key::UP => KeyCode::Up,
                godot::global::Key::RIGHT => KeyCode::Right,
                godot::global::Key::DOWN => KeyCode::Down,
                godot::global::Key::PAGEUP => KeyCode::PageUp,
                godot::global::Key::PAGEDOWN => KeyCode::PageDown,
                godot::global::Key::SHIFT => KeyCode::Modifier(ModifierKeyCode::LeftShift),
                godot::global::Key::CTRL => KeyCode::Modifier(ModifierKeyCode::LeftControl),
                godot::global::Key::META => KeyCode::Modifier(ModifierKeyCode::LeftMeta),
                godot::global::Key::ALT => KeyCode::Modifier(ModifierKeyCode::LeftAlt),
                godot::global::Key::CAPSLOCK => KeyCode::CapsLock,
                godot::global::Key::NUMLOCK => KeyCode::NumLock,
                godot::global::Key::SCROLLLOCK => KeyCode::ScrollLock,
                godot::global::Key::F1 => KeyCode::F(1),
                godot::global::Key::F2 => KeyCode::F(2),
                godot::global::Key::F3 => KeyCode::F(3),
                godot::global::Key::F4 => KeyCode::F(4),
                godot::global::Key::F5 => KeyCode::F(5),
                godot::global::Key::F6 => KeyCode::F(6),
                godot::global::Key::F7 => KeyCode::F(7),
                godot::global::Key::F8 => KeyCode::F(8),
                godot::global::Key::F9 => KeyCode::F(9),
                godot::global::Key::F10 => KeyCode::F(10),
                godot::global::Key::F11 => KeyCode::F(11),
                godot::global::Key::F12 => KeyCode::F(12),
                godot::global::Key::F13 => KeyCode::F(13),
                godot::global::Key::F14 => KeyCode::F(14),
                godot::global::Key::F15 => KeyCode::F(15),
                godot::global::Key::F16 => KeyCode::F(16),
                godot::global::Key::F17 => KeyCode::F(17),
                godot::global::Key::F18 => KeyCode::F(18),
                godot::global::Key::F19 => KeyCode::F(19),
                godot::global::Key::F20 => KeyCode::F(20),
                godot::global::Key::F21 => KeyCode::F(21),
                godot::global::Key::F22 => KeyCode::F(22),
                godot::global::Key::F23 => KeyCode::F(23),
                godot::global::Key::F24 => KeyCode::F(24),
                godot::global::Key::F25 => KeyCode::F(25),
                godot::global::Key::F26 => KeyCode::F(26),
                godot::global::Key::F27 => KeyCode::F(27),
                godot::global::Key::F28 => KeyCode::F(28),
                godot::global::Key::F29 => KeyCode::F(29),
                godot::global::Key::F30 => KeyCode::F(30),
                godot::global::Key::F31 => KeyCode::F(31),
                godot::global::Key::F32 => KeyCode::F(32),
                godot::global::Key::F33 => KeyCode::F(33),
                godot::global::Key::F34 => KeyCode::F(34),
                godot::global::Key::F35 => KeyCode::F(35),
                godot::global::Key::KP_MULTIPLY => KeyCode::Char('*'),
                godot::global::Key::KP_DIVIDE => KeyCode::Char('/'),
                godot::global::Key::KP_SUBTRACT => KeyCode::Char('-'),
                godot::global::Key::KP_PERIOD => KeyCode::Char('.'),
                godot::global::Key::KP_ADD => KeyCode::Char('+'),
                godot::global::Key::KP_0 | godot::global::Key::KEY_0 => KeyCode::Char('0'),
                godot::global::Key::KP_1 | godot::global::Key::KEY_1 => KeyCode::Char('1'),
                godot::global::Key::KP_2 | godot::global::Key::KEY_2 => KeyCode::Char('2'),
                godot::global::Key::KP_3 | godot::global::Key::KEY_3 => KeyCode::Char('3'),
                godot::global::Key::KP_4 | godot::global::Key::KEY_4 => KeyCode::Char('4'),
                godot::global::Key::KP_5 | godot::global::Key::KEY_5 => KeyCode::Char('5'),
                godot::global::Key::KP_6 | godot::global::Key::KEY_6 => KeyCode::Char('6'),
                godot::global::Key::KP_7 | godot::global::Key::KEY_7 => KeyCode::Char('7'),
                godot::global::Key::KP_8 | godot::global::Key::KEY_8 => KeyCode::Char('8'),
                godot::global::Key::KP_9 | godot::global::Key::KEY_9 => KeyCode::Char('9'),
                godot::global::Key::MENU => KeyCode::Menu,
                godot::global::Key::HYPER => KeyCode::Modifier(ModifierKeyCode::LeftHyper),
                godot::global::Key::STOP => KeyCode::Media(MediaKeyCode::Stop),
                godot::global::Key::VOLUMEDOWN => KeyCode::Media(MediaKeyCode::LowerVolume),
                godot::global::Key::VOLUMEMUTE => KeyCode::Media(MediaKeyCode::MuteVolume),
                godot::global::Key::VOLUMEUP => KeyCode::Media(MediaKeyCode::RaiseVolume),
                godot::global::Key::MEDIAPLAY => KeyCode::Media(MediaKeyCode::Play),
                godot::global::Key::MEDIASTOP => KeyCode::Media(MediaKeyCode::Stop),
                godot::global::Key::MEDIAPREVIOUS => KeyCode::Media(MediaKeyCode::TrackPrevious),
                godot::global::Key::MEDIANEXT => KeyCode::Media(MediaKeyCode::TrackNext),
                godot::global::Key::MEDIARECORD => KeyCode::Media(MediaKeyCode::Record),
                godot::global::Key::SPACE => KeyCode::Char(' '),
                godot::global::Key::EXCLAM => KeyCode::Char('!'),
                godot::global::Key::QUOTEDBL => KeyCode::Char('"'),
                godot::global::Key::NUMBERSIGN => KeyCode::Char('#'),
                godot::global::Key::DOLLAR => KeyCode::Char('$'),
                godot::global::Key::PERCENT => KeyCode::Char('%'),
                godot::global::Key::AMPERSAND => KeyCode::Char('&'),
                godot::global::Key::APOSTROPHE => KeyCode::Char('\''),
                godot::global::Key::PARENLEFT => KeyCode::Char('('),
                godot::global::Key::PARENRIGHT => KeyCode::Char(')'),
                godot::global::Key::ASTERISK => KeyCode::Char('*'),
                godot::global::Key::PLUS => KeyCode::Char('+'),
                godot::global::Key::COMMA => KeyCode::Char(','),
                godot::global::Key::MINUS => KeyCode::Char('-'),
                godot::global::Key::PERIOD => KeyCode::Char('.'),
                godot::global::Key::SLASH => KeyCode::Char('/'),
                godot::global::Key::COLON => KeyCode::Char(':'),
                godot::global::Key::SEMICOLON => KeyCode::Char(';'),
                godot::global::Key::LESS => KeyCode::Char('<'),
                godot::global::Key::EQUAL => KeyCode::Char('='),
                godot::global::Key::GREATER => KeyCode::Char('>'),
                godot::global::Key::QUESTION => KeyCode::Char('?'),
                godot::global::Key::AT => KeyCode::Char('@'),
                godot::global::Key::A => KeyCode::Char('a'),
                godot::global::Key::B => KeyCode::Char('b'),
                godot::global::Key::C => KeyCode::Char('c'),
                godot::global::Key::D => KeyCode::Char('d'),
                godot::global::Key::E => KeyCode::Char('e'),
                godot::global::Key::F => KeyCode::Char('f'),
                godot::global::Key::G => KeyCode::Char('g'),
                godot::global::Key::H => KeyCode::Char('h'),
                godot::global::Key::I => KeyCode::Char('i'),
                godot::global::Key::J => KeyCode::Char('j'),
                godot::global::Key::K => KeyCode::Char('k'),
                godot::global::Key::L => KeyCode::Char('l'),
                godot::global::Key::M => KeyCode::Char('m'),
                godot::global::Key::N => KeyCode::Char('n'),
                godot::global::Key::O => KeyCode::Char('o'),
                godot::global::Key::P => KeyCode::Char('p'),
                godot::global::Key::Q => KeyCode::Char('q'),
                godot::global::Key::R => KeyCode::Char('r'),
                godot::global::Key::S => KeyCode::Char('s'),
                godot::global::Key::T => KeyCode::Char('t'),
                godot::global::Key::U => KeyCode::Char('u'),
                godot::global::Key::V => KeyCode::Char('v'),
                godot::global::Key::W => KeyCode::Char('w'),
                godot::global::Key::X => KeyCode::Char('x'),
                godot::global::Key::Y => KeyCode::Char('y'),
                godot::global::Key::Z => KeyCode::Char('z'),
                godot::global::Key::BRACKETLEFT => KeyCode::Char('['),
                godot::global::Key::BACKSLASH => KeyCode::Char('\\'),
                godot::global::Key::BRACKETRIGHT => KeyCode::Char(']'),
                godot::global::Key::ASCIICIRCUM => KeyCode::Char('^'),
                godot::global::Key::UNDERSCORE => KeyCode::Char('_'),
                godot::global::Key::QUOTELEFT => KeyCode::Char('`'),
                godot::global::Key::BRACELEFT => KeyCode::Char('{'),
                godot::global::Key::BAR => KeyCode::Char('|'),
                godot::global::Key::BRACERIGHT => KeyCode::Char('}'),
                godot::global::Key::ASCIITILDE => KeyCode::Char('~'),
                godot::global::Key::YEN => KeyCode::Char('¥'),
                godot::global::Key::SECTION => KeyCode::Char('§'),
                _ => KeyCode::Null,
            };
            let mut modifiers = KeyModifiers::empty();
            if input_event_key.is_shift_pressed() {
                modifiers |= KeyModifiers::SHIFT;
            }
            if input_event_key.is_command_or_control_pressed() {
                modifiers |= KeyModifiers::CONTROL;
            }
            if input_event_key.is_alt_pressed() {
                modifiers |= KeyModifiers::ALT;
            }
            if input_event_key.is_meta_pressed() {
                modifiers |= KeyModifiers::META;
            }
            let kind = if input_event_key.is_pressed() {
                KeyEventKind::Press
            } else if input_event_key.is_released() {
                KeyEventKind::Release
            } else {
                KeyEventKind::Repeat
            };
            let state = KeyEventState::NONE;
            self.event(crossterm::event::Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
            }));
        }
    }

    #[instrument(skip_all)]
    fn exit_tree(&mut self) {
        ratatui::restore();
    }
}

#[godot_api]
impl Terminal {
    #[signal]
    fn init();
    #[signal]
    fn pause(state: bool);

    #[func]
    fn _on_init(&mut self) {
        self.effect = fx::coalesce((1000, Interpolation::Linear));
        self.initialized = true;
    }

    #[func]
    fn _on_pause(&mut self, state: bool) {
        self.paused = state;
    }
}

impl Terminal {
    #[instrument(skip(self))]
    fn event(&mut self, event: crossterm::event::Event) {
        tracing::debug!("New Event");
        match event {
            event::Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.terminal_command.insert(self.cursor_x.byte(), c);
                self.cursor_x.incr_by_char(c);
            }
            event::Event::Key(KeyEvent {
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let Some(char) = self
                    .terminal_command
                    .chars()
                    .nth(self.cursor_x.character().saturating_sub(1))
                {
                    self.cursor_x.decr_by_char(char);
                }
            }
            event::Event::Key(KeyEvent {
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let Some(char) = self.terminal_command.chars().nth(self.cursor_x.character()) {
                    self.cursor_x.incr_by_char(char);
                }
            }
            event::Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.cursor_x = CursorX::default();
                self.command();
            }
            event::Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.cursor_x = CursorX::default();
                self.terminal_command.clear();
            }
            event::Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let character_x = self.cursor_x.character();
                if character_x == 0 {
                    return;
                }
                if let Some((index, char)) = self
                    .terminal_command
                    .char_indices()
                    .nth(character_x.saturating_sub(1))
                {
                    self.terminal_command.remove(index);
                    self.cursor_x.decr_by_char(char);
                }
            }
            _ => {}
        }
    }

    fn command(&mut self) {
        if let Some(command) = shlex::split(&self.terminal_command) {
            if !self.paused {
                self.main_command(command);
            } else {
                self.pause_command(command);
            }
        }
        self.terminal_command.clear();
    }

    fn main_command(&mut self, command: Vec<String>) {
        let cli = MainCli::try_parse_from(command);
        match cli {
            Ok(command) => {
                self.clear_terminal_output();
                match command.command {
                    MainCommands::Init => {
                        if !self.initialized {
                            self.signals().init().emit();
                        }
                    }
                    MainCommands::Pause => self.signals().pause().emit(true),
                }
            }
            Err(e) => {
                self.set_terminal_output(e.render().ansi().to_string());
            }
        }
    }

    fn pause_command(&mut self, command: Vec<String>) {
        let cli = PauseCli::try_parse_from(command);
        match cli {
            Ok(command) => {
                self.clear_terminal_output();
                match command.command {
                    PauseCommands::UnPause => self.signals().pause().emit(false),
                }
            }
            Err(e) => {
                self.set_terminal_output(e.render().ansi().to_string());
            }
        }
    }

    fn set_terminal_output(&mut self, output: String) {
        self.terminal_command_output =
            Yoke::attach_to_cart(output, |f| YokeableText(f.into_text().unwrap()));
    }

    fn clear_terminal_output(&mut self) {
        self.terminal_command_output =
            Yoke::attach_to_cart(String::new(), |f| YokeableText(Text::from(f)));
    }
}
