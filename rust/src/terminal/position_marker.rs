use ratatui::prelude::*;
use ratatui::widgets::Widget;
use tracing::instrument;

use crate::inverse_lerp;
use crate::terminal::{MAP_BOTTOM, MAP_LEFT, MAP_RIGHT, MAP_TOP};

pub struct PositionMarker(pub f32, pub f32);

impl Widget for PositionMarker {
    #[instrument(skip(self, buf))]
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let x = inverse_lerp(MAP_LEFT..=MAP_RIGHT, self.0).unwrap_or_default() * area.width as f32;
        let y = inverse_lerp(MAP_TOP..=MAP_BOTTOM, self.1).unwrap_or_default() * area.height as f32;

        let x = x as u16 + area.x;
        let y = y as u16 + area.y;

        buf.cell_mut((x, y))
            .map(|cell| cell.set_symbol("@").set_fg(Color::Cyan));
    }
}
