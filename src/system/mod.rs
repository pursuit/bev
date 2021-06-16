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

pub fn setup_tile(
    mut tile_sprite_handles: ResMut<TileSpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    tile_sprite_handles.handles = asset_server.load_folder("tiles").unwrap();
}
