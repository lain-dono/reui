use bevy::{prelude::*, window::PrimaryWindow};
use reui::{plugin::Recorder, Color, FillRule, Offset, Path, Rect, Rounding, Transform};

mod blendish;
mod canvas;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(reui::plugin::ReuiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, draw)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Recorder::default()));
}

fn draw(
    mut query: Query<&mut Recorder>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) -> Result {
    let window = windows.single()?;
    let scale = window.scale_factor();

    let time = time.elapsed_secs();
    let mouse = Offset::new(0.0, 0.0);
    let wsize = Offset::new(window.width(), window.height());

    for mut recorder in query.iter_mut() {
        let color = Color::bgra(0x78_DCDCDC);
        let transform = Transform::scale(scale, scale);
        let center = Offset::new(400.0, 400.0);
        let rect = Rect::from_center(center, 200.0, 100.0);

        let mut path = Path::new();
        path.rrect(rect, Rounding::same(8.0));

        recorder.clear();
        recorder.fill(&path, color, transform, FillRule::NonZero, true);

        let mut canvas = reui::Canvas::new(&mut recorder);
        canvas.push_scale(scale);

        self::canvas::render_demo(&mut canvas, mouse, wsize, time, false);
    }

    Ok(())
}
