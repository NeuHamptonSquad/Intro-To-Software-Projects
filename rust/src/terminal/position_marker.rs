use godot::obj::GdRef;
use ratatui::prelude::*;
use ratatui::widgets::Widget;
use tracing::instrument;

use crate::inverse_lerp;
use crate::terminal::tile_map::TerminalTileMap;

pub struct PositionMarker<'a>(pub f32, pub f32, pub GdRef<'a, TerminalTileMap>);

impl<'a> Widget for PositionMarker<'a> {
    #[instrument(skip(self, buf))]
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let Some(x) = inverse_lerp(self.2.get_pos_left()..=self.2.get_pos_right(), self.0) else {
            return;
        };
        let x = x * area.width as f32;

        let Some(y) = inverse_lerp(self.2.get_pos_top()..=self.2.get_pos_bottom(), self.1) else {
            return;
        };
        let y = y * area.height as f32;

        let x = x as u16 + area.x;
        let y = y as u16 + area.y;

        buf.cell_mut((x, y))
            .map(|cell| cell.set_symbol("@").set_fg(Color::Cyan));
    }
}
