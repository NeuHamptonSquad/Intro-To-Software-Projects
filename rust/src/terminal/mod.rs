use std::{io::stdout, time::Duration};

use godot::{
    classes::{InputEvent, InputEventKey, TileMap},
    prelude::*,
};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::{
        self,
        event::{
            KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MediaKeyCode,
            ModifierKeyCode,
        },
    },
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use tracing::instrument;

const TILESET: [&'static str; 28] = [
    "#", "|", "─", "└", "─", "┘", "┌", "┐", "│", "├", "┤", "┬", "┴", "┼", "░", "▉", "┃", "━", "┗",
    "┛", "┏", "┓", "┣", "┫", "┳", "┻", "╋", "▉",
];

struct TerminalTileMap(pub Buffer);

impl TerminalTileMap {
    fn new(buffer: Buffer) -> Self {
        Self(buffer)
    }

    fn apply_layer(&mut self, tilemap: &TileMap, layer: i32, layer_style: Style) {
        let tiles = tilemap.get_used_cells(layer);
        for coordinate in tiles.iter_shared() {
            if let Some(cell) = self.0.cell_mut((coordinate.x as u16, coordinate.y as u16)) {
                let cell_atlas_x = tilemap.get_cell_atlas_coords(layer, coordinate).x as usize;
                cell.set_symbol(TILESET[cell_atlas_x]);
                cell.set_style(layer_style);
            }
        }
    }
}

impl Widget for &TerminalTileMap {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let tilemap_line_iterator = self.0.content.chunks(self.0.area.width as usize);
        for (y, line) in (area.y..area.bottom()).zip(tilemap_line_iterator) {
            let line_start = buf.index_of(area.x, y);
            let target_line = &mut buf.content[line_start..line_start + line.len()];

            target_line.clone_from_slice(line);
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
/// This class is responsible for the terminal side
/// of this game.
pub struct Terminal {
    base: Base<Node>,
    tile_map: Option<TerminalTileMap>,
    terminal: DefaultTerminal,
    latest_event: crossterm::event::Event,
}

#[godot_api]
impl INode for Terminal {
    #[instrument(skip_all)]
    fn init(base: Base<Node>) -> Self {
        Self {
            tile_map: None,
            base,
            terminal: ratatui::Terminal::new(CrosstermBackend::new(stdout())).unwrap(),
            latest_event: crossterm::event::Event::FocusGained,
        }
    }

    #[instrument(skip_all)]
    fn ready(&mut self) {
        self.terminal = ratatui::init();
        self.tile_map = self
            .base()
            .try_get_node_as::<TileMap>("TileMap")
            .map(|tile_map| {
                let area = tile_map.get_used_rect();
                let mut terminal_tile_map = TerminalTileMap::new(Buffer::empty(Rect::new(
                    area.position.x as u16,
                    area.position.y as u16,
                    area.size.x as u16,
                    area.size.y as u16,
                )));

                terminal_tile_map.apply_layer(&tile_map, 0, Style::new());
                terminal_tile_map.apply_layer(&tile_map, 1, Style::new().fg(Color::Red));

                terminal_tile_map
            });
    }

    #[instrument(skip_all)]
    fn process(&mut self, delta: f64) {
        if crossterm::event::poll(Duration::from_secs(0)).unwrap_or_default() {
            self.event(crossterm::event::read().unwrap());
        }
        self.terminal
            .draw(|frame| {
                frame.render_widget(
                    Paragraph::new(format!(
                        "The terminal is working\nThat latest event was {:?}",
                        self.latest_event
                    ))
                    .wrap(Wrap { trim: true })
                    .alignment(ratatui::layout::Alignment::Center)
                    .block(
                        Block::new()
                            .title("FNAF 5: Double Vision")
                            .borders(Borders::all()),
                    ),
                    frame.area(),
                );
                let layout = Layout::new(
                    Direction::Vertical,
                    [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
                )
                .split(frame.area());
                if let Some(tile_map) = &self.tile_map {
                    frame.render_widget(tile_map, layout[1]);
                }
            })
            .unwrap();
    }

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

impl Terminal {
    #[instrument(skip(self))]
    fn event(&mut self, event: crossterm::event::Event) {
        tracing::debug!("New Event");
        self.latest_event = event;
    }
}
