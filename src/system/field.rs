use super::GameMap;
use super::Player;
use super::PlayerBundle;
use super::Position;
use super::Render;
use super::TileSpriteHandles;

use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlasBuilder};

use bevy_tilemap::prelude::*;

const CHUNK_WIDTH: u32 = 64;
const CHUNK_HEIGHT: u32 = 64;
const TILEMAP_WIDTH: i32 = CHUNK_WIDTH as i32 * 40;
const TILEMAP_HEIGHT: i32 = CHUNK_HEIGHT as i32 * 40;

pub fn load(
    mut commands: Commands,
    mut sprite_handles: ResMut<TileSpriteHandles>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    asset_server: Res<AssetServer>,
) {
    if sprite_handles.atlas_loaded {
        return;
    }

    // Lets load all our textures from our folder!
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    if let LoadState::Loaded =
        asset_server.get_group_load_state(sprite_handles.handles.iter().map(|handle| handle.id))
    {
        for handle in sprite_handles.handles.iter() {
            let texture = textures.get(handle).unwrap();
            texture_atlas_builder.add_texture(handle.clone_weak().typed::<Texture>(), &texture);
        }

        let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
        let atlas_handle = texture_atlases.add(texture_atlas);

        // These are fairly advanced configurations just to quickly showcase
        // them.
        let tilemap = Tilemap::builder()
            .dimensions(TILEMAP_WIDTH as u32, TILEMAP_HEIGHT as u32)
            .chunk_dimensions(CHUNK_WIDTH, CHUNK_HEIGHT, 1)
            .texture_dimensions(32, 32)
            .auto_chunk()
            .auto_spawn(2, 2)
            .add_layer(
                TilemapLayer {
                    kind: LayerKind::Dense,
                    ..Default::default()
                },
                0,
            )
            .texture_atlas(atlas_handle)
            .finish()
            .unwrap();

        let tilemap_components = TilemapBundle {
            tilemap,
            visible: Visible {
                is_visible: true,
                is_transparent: true,
            },
            transform: Default::default(),
            global_transform: Default::default(),
        };
        commands
            .spawn()
            .insert_bundle(OrthographicCameraBundle::new_2d());
        commands
            .spawn()
            .insert_bundle(tilemap_components)
            .insert(Timer::from_seconds(0.075, true));

        sprite_handles.atlas_loaded = true;
    }
}

pub fn build(
    mut commands: Commands,
    mut game_state: ResMut<GameMap>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Tilemap>,
) {
    if game_state.map_loaded {
        return;
    }

    for mut map in query.iter_mut() {
        let floor_sprite: Handle<Texture> =
            asset_server.get_handle("texture/tiles/generic-rpg-Slice.png");
        let wall_sprite: Handle<Texture> =
            asset_server.get_handle("texture/tiles/generic-rpg-tile02.png");
        let texture_atlas = texture_atlases.get(map.texture_atlas()).unwrap();
        let floor_idx = texture_atlas.get_texture_index(&floor_sprite).unwrap();
        let wall_idx = texture_atlas.get_texture_index(&wall_sprite).unwrap();

        // Now we fill the entire space with floors.
        let mut tiles = Vec::new();
        for y in 0..TILEMAP_HEIGHT {
            for x in 0..TILEMAP_WIDTH {
                let y = y - TILEMAP_HEIGHT / 2;
                let x = x - TILEMAP_WIDTH / 2;
                // By default tile sets the Z order at 0. Lower means that tile
                // will render lower than others. 0 is the absolute bottom
                // level which is perfect for backgrounds.
                let tile = Tile {
                    point: (x, y),
                    sprite_index: floor_idx,
                    ..Default::default()
                };
                tiles.push(tile);
            }
        }

        tiles.push(Tile {
            point: (0, 0),
            sprite_index: wall_idx,
            ..Default::default()
        });

        let dwarf_sprite: Handle<Texture> = asset_server.get_handle("texture/sprite/sensei.png");
        let dwarf_sprite_index = texture_atlas.get_texture_index(&dwarf_sprite).unwrap();
        // We add in a Z order of 1 to place the tile above the background on Z
        // order 0.
        let dwarf_tile = Tile {
            point: (2, 2),
            sprite_order: 1,
            sprite_index: dwarf_sprite_index,
            ..Default::default()
        };
        tiles.push(dwarf_tile);

        commands.spawn().insert_bundle(PlayerBundle {
            player: Player {},
            position: Position { x: 2, y: 2 },
            render: Render {
                sprite_index: dwarf_sprite_index,
                sprite_order: 1,
            },
        });

        map.insert_tiles(tiles).unwrap();
        game_state.map_loaded = true;
    }
}
