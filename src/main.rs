use bevy::{color::Srgba, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_pan)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn(Sprite {
        color: Srgba::new(0.5, 0.5, 1.0, 1.0).into(),
        custom_size: Some(Vec2::new(100.0, 100.0)),
        ..Default::default()
    });
}

fn camera_pan(
    mut camera: Query<(Entity, &mut Transform, &Camera)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_reader: MessageReader<CursorMoved>,
    mut last_pos: Local<Vec3>,

    mut commands: Commands,
) {
    let cursor = cursor_moved_reader.read().last();

    if mouse_input.pressed(MouseButton::Middle) {
        let mut delta = cursor.map_or(*last_pos, |c| c.position.extend(0.0)) - *last_pos;

        delta.x = -delta.x;
        let Ok((entity, mut transform, _)) = camera.single_mut() else {
            return;
        };
        transform.translation += delta;

        
    }

    *last_pos = cursor.map_or(*last_pos, |c| c.position.extend(0.0));
}
