use godot::{
    classes::{ITileMapLayer, TileMapLayer},
    prelude::*,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use tracing::instrument;

const TILESET: [&'static str; 28] = [
    "#", "|", "─", "└", "─", "┘", "┌", "┐", "│", "├", "┤", "┬", "┴", "┼", "░", "▉", "┃", "━", "┗",
    "┛", "┏", "┓", "┣", "┫", "┳", "┻", "╋", "▉",
];

#[derive(GodotClass)]
#[class(base=TileMapLayer)]
struct TerminalTileMapLayer {
    base: Base<TileMapLayer>,
    buffer: Buffer,
}

#[godot_api]
impl ITileMapLayer for TerminalTileMapLayer {
    #[instrument(skip_all)]
    fn init(base: Base<TileMapLayer>) -> Self {
        Self {
            base,
            buffer: Buffer::empty(Rect::new(0, 0, 1, 1)),
        }
    }

    #[instrument(skip_all)]
    fn ready(&mut self) {
        self.base_mut().set_visible(false);
    }

    #[instrument(skip(self, tiles))]
    fn update_cells(&mut self, mut tiles: Array<Vector2i>, forced_cleanup: bool) {
        let base = self.base().clone();
        let used_rect = base.get_used_rect();
        let used_rect = Rect::new(
            used_rect.position.x as u16,
            used_rect.position.y as u16,
            used_rect.size.x as u16,
            used_rect.size.y as u16,
        );
        if forced_cleanup || used_rect != self.buffer.area {
            self.buffer = Buffer::empty(used_rect);
            tiles = base.get_used_cells();
        }
        let modulate = base.get_modulate();
        let layer_style = Style::new().fg(Color::Rgb(modulate.r8(), modulate.g8(), modulate.b8()));
        for coordinate in tiles.iter_shared() {
            if let Some(cell) = self
                .buffer
                .cell_mut((coordinate.x as u16, coordinate.y as u16))
            {
                let cell_atlas_x = base.get_cell_atlas_coords(coordinate).x as usize;
                cell.set_symbol(TILESET.get(cell_atlas_x).unwrap_or(&" "));
                cell.set_style(layer_style);
            }
        }
    }
}

impl Widget for &TerminalTileMapLayer {
    #[instrument(skip_all)]
    fn render(self, mut area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.buffer.area.width <= 0 {
            return;
        }
        let tilemap_line_iterator = self.buffer.content.chunks(self.buffer.area.width as usize);
        area.x += self.buffer.area.x;
        area.y += self.buffer.area.y;
        area.width = self.buffer.area.width.min(area.width - self.buffer.area.x);
        area.height = self
            .buffer
            .area
            .height
            .min(area.height - self.buffer.area.y);
        for (y, line) in (area.y..area.bottom()).zip(tilemap_line_iterator) {
            let line_start = buf.index_of(area.x, y);
            let target_line = &mut buf.content[line_start..line_start + line.len()];

            target_line
                .iter_mut()
                .zip(line.iter())
                .filter(|(_, line_cell)| line_cell.symbol() != " ")
                .for_each(|(target_cell, line_cell)| *target_cell = line_cell.clone());
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct TerminalTileMap {
    base: Base<Node>,
    layers: Vec<Gd<TerminalTileMapLayer>>,
}

#[godot_api]
impl INode for TerminalTileMap {
    #[instrument(skip_all)]
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            layers: Vec::new(),
        }
    }

    #[instrument(skip_all)]
    fn ready(&mut self) {
        self.layers.clear();
        self.layers.extend(
            self.base()
                .get_children()
                .iter_shared()
                .flat_map(|node| node.try_cast::<TerminalTileMapLayer>()),
        );
    }
}

impl TerminalTileMap {
    #[instrument(skip_all)]
    pub fn get_used_rect(&self) -> Rect {
        self.layers.iter().fold(Rect::ZERO, |rect, layer| {
            rect.union(layer.bind().buffer.area)
        })
    }
}

impl Widget for &TerminalTileMap {
    #[instrument(skip_all)]
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for layer in &self.layers {
            layer.bind().render(area, buf)
        }
    }
}
