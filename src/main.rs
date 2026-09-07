use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite_render::{MeshMaterial2d, SpriteMeshMaterial},
    window::PrimaryWindow,
};

const FRAME_COUNT: usize = 4;
const FRAME_SIZE: u32 = 32;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (800, 400).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.035, 0.035, 0.035)))
        .init_resource::<Playback>()
        .add_systems(Startup, setup)
        .add_systems(Update, (controls, animate).chain())
        .run();
}

#[derive(Component)]
struct AutomaticSprite;
#[derive(Component)]
struct DirectMaterial;

#[derive(Resource)]
struct Playback {
    timer: Timer,
    frame: usize,
    paused: bool,
    mode: u8,
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            frame: 0,
            paused: false,
            mode: 3,
        }
    }
}

// Every frame has an identical opaque silhouette. Only the interior stripe moves.
// There are no empty frames, external assets, loaders, scenes, or physics.
fn atlas_image() -> Image {
    let width = FRAME_SIZE * FRAME_COUNT as u32;
    let mut pixels = vec![0; (width * FRAME_SIZE * 4) as usize];
    for frame in 0..FRAME_COUNT as u32 {
        for y in 4..28 {
            for x in 4..28 {
                let color = if (6 + frame * 5..9 + frame * 5).contains(&x) {
                    [255, 255, 255, 255]
                } else {
                    [50, 185, 230, 255]
                };
                let offset = ((y * width + frame * FRAME_SIZE + x) * 4) as usize;
                pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }
    Image::new(
        Extent3d {
            width,
            height: FRAME_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SpriteMeshMaterial>>,
) {
    commands.spawn(Camera2d);
    let layout =
        TextureAtlasLayout::from_grid(UVec2::splat(FRAME_SIZE), FRAME_COUNT as u32, 1, None, None);
    let sprite = Sprite {
        image: images.add(atlas_image()),
        texture_atlas: Some(TextureAtlas {
            layout: layouts.add(layout.clone()),
            index: 0,
        }),
        custom_size: Some(Vec2::splat(192.0)),
        ..default()
    };
    let mut material = SpriteMeshMaterial::from_sprite(sprite.clone());
    // from_sprite does not resolve the atlas layout. Match SpriteMeshPlugin.
    material.texture_atlas_layout = Some(layout);
    material.texture_atlas_index = 0;

    commands.spawn((
        AutomaticSprite,
        sprite,
        Transform::from_xyz(-180.0, 0.0, 0.0),
    ));
    // No Sprite component here: the automatic material cache must not replace this handle.
    commands.spawn((
        DirectMaterial,
        Mesh2d(meshes.add(Rectangle::from_size(Vec2::ONE))),
        MeshMaterial2d(materials.add(material)),
        Transform::from_xyz(180.0, 0.0, 0.0),
    ));
}

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut playback: ResMut<Playback>,
    mut subjects: Query<
        (&mut Visibility, &mut Transform, Has<AutomaticSprite>),
        Or<(With<AutomaticSprite>, With<DirectMaterial>)>,
    >,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    for (key, mode) in [
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
    ] {
        if keys.just_pressed(key) {
            playback.mode = mode;
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        playback.mode = playback.mode % 3 + 1;
    }
    if keys.just_pressed(KeyCode::Space) {
        playback.paused = !playback.paused;
    }
    if !playback.is_changed() {
        return;
    }
    for (mut visibility, mut transform, automatic) in &mut subjects {
        let show = playback.mode == 3 || (playback.mode == 1) == automatic;
        *visibility = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation.x = if playback.mode == 3 {
            if automatic { -180.0 } else { 180.0 }
        } else {
            0.0
        };
    }
    let mode = match playback.mode {
        1 => "Sprite (automatic material)",
        2 => "SpriteMeshMaterial (stable handle)",
        _ => "LEFT: Sprite | RIGHT: SpriteMeshMaterial",
    };
    window.title = format!(
        "{mode} | 1/2/3 or Tab | Space: pause | {}",
        if playback.paused { "PAUSED" } else { "5 fps" }
    );
}

fn animate(
    time: Res<Time>,
    mut playback: ResMut<Playback>,
    mut sprites: Query<&mut Sprite, With<AutomaticSprite>>,
    direct: Query<&MeshMaterial2d<SpriteMeshMaterial>, With<DirectMaterial>>,
    mut materials: ResMut<Assets<SpriteMeshMaterial>>,
) {
    if playback.paused || !playback.timer.tick(time.delta()).just_finished() {
        return;
    }
    playback.frame = (playback.frame + 1) % FRAME_COUNT;
    for mut sprite in &mut sprites {
        // A: Bevy's Changed<Sprite> system selects/creates a material handle.
        sprite.texture_atlas.as_mut().unwrap().index = playback.frame;
    }
    for handle in &direct {
        // B: Update the same material during Update, before AssetEventSystems.
        let mut material = materials.get_mut(&handle.0).unwrap();
        material.texture_atlas_index = playback.frame;
        material.texture_atlas.as_mut().unwrap().index = playback.frame;
    }
}

#[cfg(test)]
mod tests;
