use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game: ResMut<super::GameCamera>,
) {
    let texture_handle = asset_server.load("sprite/icon.png");
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(texture_handle.clone().into()),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });

    game.player.entity = Some(
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(texture_handle.into()),
                transform: Transform {
                    translation: Vec3::new(120.0, 0.0, 100.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(super::MainChar)
            .id(),
    );
}
