use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game: ResMut<super::GameCamera>,
) {
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(asset_server.load("sprite/red hat boy/Dead (1).png").into()),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });

    game.player.entity = Some(
        commands
            .spawn_bundle(SpriteBundle {
                material: materials
                    .add(asset_server.load("sprite/red hat boy/Idle (1).png").into()),
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(super::MainChar)
            .id(),
    );
}
