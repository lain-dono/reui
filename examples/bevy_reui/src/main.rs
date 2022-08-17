use bevy::{prelude::*, window::WindowId};
use reui::{plugin::Recorder, Color, FillRule, Offset, Path, Rect, Rounding, Transform};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(reui::plugin::ReuiPlugin)
        .add_startup_system(setup)
        .add_system(draw)
        .run();
}

fn setup(mut commands: Commands, windows: Res<Windows>) {
    let factor = windows.scale_factor(WindowId::primary()) as f32;

    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(Recorder::default());
}

fn draw(mut query: Query<&mut Recorder>, windows: Res<Windows>) {
    let scale = windows.scale_factor(WindowId::primary()) as f32;
    for mut recorder in query.iter_mut() {
        let color = Color::bgra(0x78_DCDCDC);
        let transform = Transform::identity();
        let center = Offset::new(400.0, 400.0);
        let rect = Rect::from_center(center, 200.0, 100.0);

        let mut path = Path::new();
        path.rrect(rect, Rounding::same(8.0));

        recorder.clear();
        recorder.fill(&path, color, transform, FillRule::NonZero, true);
    }
}
