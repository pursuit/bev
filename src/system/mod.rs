use std::sync::Mutex;

use crate::pursuit::api::mortalkin::{GameNotif, PlayGamePayload};

use bevy::prelude::*;
use bevy::render::camera::Camera;

use bevy_tilemap::prelude::*;

pub mod char_creation;
pub mod char_selection;
pub mod field;
pub mod login;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    CharSelectionMenu,
    CharCreationMenu,
    Field,
}

pub struct RequestSender {
    pub tx: Mutex<futures::channel::mpsc::UnboundedSender<PlayGamePayload>>,
}

pub struct ResponseReceiver {
    pub rx: Mutex<futures::channel::mpsc::UnboundedReceiver<GameNotif>>,
}

pub struct Token {
    pub token: String,
}

pub struct UserCharacters {
    pub characters: Vec<Character>,
}

#[derive(Clone)]
pub struct Character {
    pub id: u32,
    pub name: String,
    pub position: Option<Position>,
}

#[derive(Default, Copy, Clone, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

#[derive(Default, Clone)]
pub struct TileSpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

#[derive(Default, Clone)]
pub struct GameMap {
    map_loaded: bool,
}

impl GameMap {
    fn try_move_player(
        &mut self,
        position: &mut Position,
        camera_translation: &mut Vec3,
        delta_xy: (i32, i32),
    ) {
        position.x = position.x + delta_xy.0;
        position.y = position.y + delta_xy.1;
        camera_translation.x = camera_translation.x + (delta_xy.0 as f32 * 32.);
        camera_translation.y = camera_translation.y + (delta_xy.1 as f32 * 32.);
    }
}

pub fn setup_tile(
    mut tile_sprite_handles: ResMut<TileSpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    tile_sprite_handles.handles = asset_server.load_folder("texture").unwrap();
}

#[derive(Default)]
pub struct Player {
    id: u32,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    position: Position,
    render: Render,
}

#[derive(Default)]
pub struct Render {
    sprite_index: usize,
    sprite_order: usize,
}

fn move_sprite(
    map: &mut Tilemap,
    previous_position: Position,
    position: Position,
    render: &Render,
) {
    // We need to first remove where we were prior.
    map.clear_tile((previous_position.x, previous_position.y), 1)
        .unwrap();
    // We then need to update where we are going!
    let tile = Tile {
        point: (position.x, position.y),
        sprite_index: render.sprite_index,
        sprite_order: render.sprite_order,
        ..Default::default()
    };
    map.insert_tile(tile).unwrap();
}

pub fn character_movement(
    current_char: Res<Character>,
    mut game_state: ResMut<GameMap>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut map_query: Query<(&mut Tilemap, &mut Timer)>,
    mut player_query: Query<(&mut Position, &Render, &Player)>,
    mut camera_query: Query<(&Camera, &mut Transform)>,
) {
    if !game_state.map_loaded {
        return;
    }

    for (mut map, mut timer) in map_query.iter_mut() {
        timer.tick(time.delta());
        if !timer.finished() {
            continue;
        }

        for (mut position, render, player) in player_query.iter_mut() {
            if player.id != current_char.id {
                move_sprite(&mut map, *position, *position, render);
                continue
            }

            for key in keyboard_input.get_pressed() {
                for (_camera, mut camera_transform) in camera_query.iter_mut() {
                    // First we need to store our very current position.
                    let previous_position = *position;

                    // Of course we need to control where we are going to move our
                    // dwarf friend.
                    use KeyCode::*;
                    match key {
                        W | Numpad8 | Up | K => {
                            game_state.try_move_player(
                                &mut position,
                                &mut camera_transform.translation,
                                (0, 1),
                            );
                        }
                        A | Numpad4 | Left | H => {
                            game_state.try_move_player(
                                &mut position,
                                &mut camera_transform.translation,
                                (-1, 0),
                            );
                        }
                        S | Numpad2 | Down | J => {
                            game_state.try_move_player(
                                &mut position,
                                &mut camera_transform.translation,
                                (0, -1),
                            );
                        }
                        D | Numpad6 | Right | L => {
                            game_state.try_move_player(
                                &mut position,
                                &mut camera_transform.translation,
                                (1, 0),
                            );
                        }

                        Numpad9 | U => game_state.try_move_player(
                            &mut position,
                            &mut camera_transform.translation,
                            (1, 1),
                        ),
                        Numpad3 | M => game_state.try_move_player(
                            &mut position,
                            &mut camera_transform.translation,
                            (1, -1),
                        ),
                        Numpad1 | N => game_state.try_move_player(
                            &mut position,
                            &mut camera_transform.translation,
                            (-1, -1),
                        ),
                        Numpad7 | Y => game_state.try_move_player(
                            &mut position,
                            &mut camera_transform.translation,
                            (-1, 1),
                        ),

                        _ => {}
                    }

                    // Finally now we will move the sprite! ... Provided he had
                    // moved!
                    move_sprite(&mut map, previous_position, *position, render);
                }
            }
        }
    }
}

pub fn incoming_notif(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    response_receiver: Res<ResponseReceiver>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    current_char: Res<Character>,
    mut query: Query<&mut Tilemap>,
) {
    let resp = response_receiver.rx.lock().unwrap().try_next();
    match resp {
        Ok(Some(conn_resp)) => {
            for chars in conn_resp.characters.into_iter() {
                if chars.id == current_char.id {
                    continue;
                }

                for map in query.iter_mut() {
                    let texture_atlas = texture_atlases.get(map.texture_atlas()).unwrap();
                    let dwarf_sprite: Handle<Texture> =
                        asset_server.get_handle("texture/sprite/sensei.png");
                    let dwarf_sprite_index =
                        texture_atlas.get_texture_index(&dwarf_sprite).unwrap();

                    commands.spawn().insert_bundle(PlayerBundle {
                        player: Player {
                            id: chars.id,
                        },
                        position: Position { x: 2, y: 2 },
                        render: Render {
                            sprite_index: dwarf_sprite_index,
                            sprite_order: 1,
                        },
                    });
                }
            }
        }
        Ok(None) => {}
        Err(_) => {}
    }
}
