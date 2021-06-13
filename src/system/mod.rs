use std::sync::Mutex;

use crate::pursuit::api::mortalkin::{GameNotif, PlayGamePayload};

use bevy::prelude::*;

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

#[derive(Clone)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

pub struct MainChar;

#[derive(Default)]
pub struct Player {
    entity: Option<Entity>,
    i: usize,
    j: usize,
}

#[derive(Default)]
pub struct GameCamera {
    camera_should_focus: Vec3,
    camera_is_focus: Vec3,
    player: Player,
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

const BOARD_SIZE_I: usize = 14;
const BOARD_SIZE_J: usize = 21;

const RESET_FOCUS: [f32; 3] = [
    BOARD_SIZE_I as f32 / 2.0,
    0.0,
    BOARD_SIZE_J as f32 / 2.0 - 0.5,
];

pub fn setup_camera(mut commands: Commands, mut game: ResMut<GameCamera>) {
    commands.spawn_bundle(UiCameraBundle::default());

    game.camera_should_focus = Vec3::from(RESET_FOCUS);
    game.camera_is_focus = game.camera_should_focus;

    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.looking_at(game.camera_is_focus, Vec3::Y);
    commands.spawn_bundle(camera);
}

pub fn move_player(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut game: ResMut<GameCamera>,
    mut transforms: Query<&mut Transform, With<MainChar>>,
) {
    let mut moved = false;
    if keyboard_input.just_pressed(KeyCode::Up) {
        game.player.j += 30;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::Down) {
        if game.player.j > 30 {
            game.player.j -= 30;
            moved = true;
        }
    }
    if keyboard_input.just_pressed(KeyCode::Right) {
        game.player.i += 30;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::Left) {
        if game.player.i > 30 {
            game.player.i -= 30;
            moved = true;
        }
    }

    if moved {
        *transforms.get_mut(game.player.entity.unwrap()).unwrap() = Transform {
            translation: Vec3::new(game.player.i as f32, game.player.j as f32, 0.0),
            ..Default::default()
        };
    }
}
