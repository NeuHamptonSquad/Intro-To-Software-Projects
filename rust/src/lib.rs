use godot::{
    classes::{ISprite2D, Sprite2D},
    prelude::*,
};
use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod tracing_godot;

struct FnafDoubleVisionExtension;

#[gdextension]
unsafe impl ExtensionLibrary for FnafDoubleVisionExtension {
    fn on_level_init(level: InitLevel) {
        if matches!(level, InitLevel::Core) {
            color_eyre::config::HookBuilder::new()
                .theme(color_eyre::config::Theme::default())
                .install()
                .unwrap();

            tracing_subscriber::registry()
                .with(ErrorLayer::default())
                .with(
                    EnvFilter::try_from_default_env()
                        .or_else(|_| EnvFilter::try_new("info"))
                        .unwrap(),
                )
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(tracing_godot::GodotMakeWriter)
                        .with_ansi(false),
                )
                .init();
        }
    }
}

#[derive(GodotClass)]
#[class(base=Sprite2D)]
struct Player {
    speed: f64,
    angular_speed: f64,

    base: Base<Sprite2D>,
}

#[godot_api]
impl ISprite2D for Player {
    #[instrument(skip_all)]
    fn init(base: Base<Sprite2D>) -> Self {
        tracing::info!("Hello, world!"); // Prints to the Godot console

        Self {
            speed: 400.0,
            angular_speed: std::f64::consts::PI,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // GDScript code:
        //
        // rotation += angular_speed * delta
        // var velocity = Vector2.UP.rotated(rotation) * speed
        // position += velocity * delta

        let radians = (self.angular_speed * delta) as f32;
        self.base_mut().rotate(radians);

        let rotation = self.base().get_rotation();
        let velocity = Vector2::UP.rotated(rotation) * self.speed as f32;
        self.base_mut().translate(velocity * delta as f32);

        // or verbose:
        // let this = self.base_mut();
        // this.set_position(
        //     this.position() + velocity * delta as f32
        // );
    }
}
