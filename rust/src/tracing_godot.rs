use std::io::Write;

use godot::builtin::Variant;
use tracing::Level;
use tracing_subscriber::fmt::MakeWriter;

pub struct GodotMakeWriter;

impl<'a> MakeWriter<'a> for GodotMakeWriter {
    type Writer = GodotWriter;

    fn make_writer(&'a self) -> Self::Writer {
        GodotWriter(Level::INFO)
    }
}

pub struct GodotWriter(Level);

impl Write for GodotWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf = unsafe { std::str::from_utf8_unchecked(buf) };
        match self.0 {
            Level::INFO => godot::global::print(&[Variant::from(buf)]),
            Level::WARN => godot::global::push_warning(&[Variant::from(buf)]),
            Level::ERROR => godot::global::push_error(&[Variant::from(buf)]),
            _ => godot::global::print_verbose(&[Variant::from(buf)]),
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
