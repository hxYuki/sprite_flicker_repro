use super::*;
use bevy::{
    asset::AssetPlugin, ecs::message::MessageCursor, shader::Shader,
    sprite_render::SpriteMeshPlugin,
};

#[test]
fn reproduces_material_event_gap_only_on_sprite_path() {
    let mut app = App::new();
    app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
    app.init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<Shader>()
        .init_asset::<TextureAtlasLayout>()
        .add_plugins(SpriteMeshPlugin);
    app.init_resource::<Playback>();
    app.world_mut().run_system_cached(setup).unwrap();
    for _ in 0..5 {
        app.update();
    }

    let mut reader = MessageCursor::<AssetEvent<SpriteMeshMaterial>>::default();
    for frame in [1, 2, 3, 0, 1] {
        let world = app.world_mut();
        for mut sprite in world
            .query_filtered::<&mut Sprite, With<AutomaticSprite>>()
            .iter_mut(world)
        {
            sprite.texture_atlas.as_mut().unwrap().index = frame;
        }
        let direct = world
            .query_filtered::<&MeshMaterial2d<SpriteMeshMaterial>, With<DirectMaterial>>()
            .single(world)
            .unwrap()
            .0
            .clone();
        world
            .resource_mut::<Assets<SpriteMeshMaterial>>()
            .get_mut(&direct)
            .unwrap()
            .texture_atlas_index = frame;
        app.update();

        let world = app.world_mut();
        let automatic = world
            .query_filtered::<&MeshMaterial2d<SpriteMeshMaterial>, With<AutomaticSprite>>()
            .single(world)
            .unwrap()
            .id();
        let events: Vec<_> = reader
            .read(world.resource::<Messages<AssetEvent<SpriteMeshMaterial>>>())
            .copied()
            .collect();
        let automatic_announced = events
            .iter()
            .any(|event| matches!(event, AssetEvent::Added { id } if *id == automatic));
        let direct_announced = events
            .iter()
            .any(|event| matches!(event, AssetEvent::Modified { id } if *id == direct.id()));
        println!(
            "frame {frame}: Sprite new-material event={automatic_announced}, direct-material update event={direct_announced}"
        );
        assert!(
            !automatic_announced,
            "This version no longer reproduces the Sprite event gap"
        );
        assert!(
            direct_announced,
            "Direct material changes should reach extraction this frame"
        );
        for _ in 0..4 {
            app.update();
        }
    }
}
