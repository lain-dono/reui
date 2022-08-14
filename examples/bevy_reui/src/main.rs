use bevy::{prelude::*, window::WindowId};
use reui::{Color, FillRule, Offset, Paint, Path, Recorder, Rect, Rounding, Transform};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(reui::integration::ReuiPlugin)
        .add_startup_system(setup)
        .add_system(draw)
        .run();
}

#[derive(Component)]
pub struct ReuiView {
    pub recorder: Recorder,
}

fn setup(mut commands: Commands) {
    // ui camera
    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert(Recorder::new());
}

fn draw(windows: Res<Windows>, mut query: Query<&mut Recorder>) {
    let scale_factor = windows.scale_factor(WindowId::primary()) as f32;
    for mut recorder in query.iter_mut() {
        let color = Color::hex(0x78_DCDCDC);
        let paint = Paint::fill_non_zero(color);
        let transform = Transform::identity();

        let center = Offset::new(400.0, 400.0);
        let rect = Rect::from_center(center, 200.0, 100.0);

        let mut path = Path::new();
        path.rrect(rect, Rounding::same(8.0));

        recorder.clear();
        recorder.fill_path(
            path.iter(),
            &paint,
            transform,
            scale_factor,
            FillRule::NonZero,
        );
    }
}
