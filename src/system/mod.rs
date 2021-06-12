use std::sync::Mutex;

use crate::pursuit::api::mortalkin::{GameNotif, PlayGamePayload};

use bevy::prelude::*;

pub mod char_creation;
pub mod char_selection;
pub mod login;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    CharSelectionMenu,
    CharCreationMenu,
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

pub struct Character {
    pub id: i64,
    pub name: String,
    pub position: Option<Position>,
}

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
