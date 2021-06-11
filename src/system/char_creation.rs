use bevy::prelude::*;

use super::AppState;
use super::ButtonMaterials;
use super::Token;
use super::UserCharacters;

use crate::pursuit::api::mortalkin::{Character, CreateCharacterPayload};

use std::sync::mpsc;
use std::sync::Mutex;

pub struct NameText;
pub struct CreateButtonText;
pub struct CreateFormUI;
pub struct Action {
    action: u32,
}

impl Action {
    pub fn new() -> Self {
        Self { action: 0 }
    }
}

pub struct RequestSender {
    pub tx: Mutex<mpsc::Sender<CreateCharacterPayload>>,
}

pub struct ResponseReceiver {
    pub rx: Mutex<mpsc::Receiver<Result<tonic::Response<Character>, tonic::Status>>>,
}

pub fn setup_create_form(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    button_materials: Res<ButtonMaterials>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .insert(CreateFormUI)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        align_self: AlignSelf::FlexEnd,
                        ..Default::default()
                    },
                    // Use `Text` directly
                    text: Text {
                        // Construct a `Vec` of `TextSection`s
                        sections: vec![
                            TextSection {
                                value: "Name: ".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.otf"),
                                    font_size: 60.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Medium.otf"),
                                    font_size: 60.0,
                                    color: Color::GOLD,
                                },
                            },
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(CreateFormUI)
                .insert(NameText);

            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        // center button
                        margin: Rect::all(Val::Auto),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: button_materials.normal.clone(),
                    ..Default::default()
                })
                .insert(CreateFormUI)
                .with_children(|pparent| {
                    pparent
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                "Button",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.otf"),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        })
                        .insert(CreateFormUI)
                        .insert(CreateButtonText);
                });
        });
}

pub fn create_button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text, (With<CreateButtonText>, Without<NameText>)>,
    mut action: ResMut<Action>,
    request_sender: Res<RequestSender>,
    token: Res<Token>,
    user_query: Query<&Text, With<NameText>>,
) {
    if action.action == 1 {
        return;
    }

    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                action.action = 1;
                text.sections[0].value = "Creating".to_string();
                *material = button_materials.pressed.clone();

                let name = user_query.single().unwrap().sections[1].value.clone();

                request_sender
                    .tx
                    .lock()
                    .unwrap()
                    .send(CreateCharacterPayload {
                        token: token.token.clone(),
                        name: name,
                    })
                    .unwrap();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                text.sections[0].value = "Create".to_string();
                *material = button_materials.normal.clone();
            }
        }
    }
}

pub fn cleanup(mut commands: Commands, q: Query<Entity, With<CreateFormUI>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

pub fn input_event_system(
    action: Res<Action>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut name_query: Query<&mut Text, With<NameText>>,
) {
    if action.action == 1 {
        return;
    }

    for event in char_input_events.iter() {
        let name = &mut name_query.single_mut().unwrap().sections[1];
        if event.char == '\x08' {
            name.value.pop();
        } else {
            name.value.push(event.char);
        }
    }
}

pub fn submit_system(
    mut action: ResMut<Action>,
    mut app_state: ResMut<State<AppState>>,
    response_receiver: ResMut<ResponseReceiver>,
    mut user_characters: ResMut<UserCharacters>,
) {
    if action.action != 1 {
        return;
    }

    let resp = response_receiver.rx.lock().unwrap().try_recv();
    match resp {
        Ok(conn_resp) => {
            action.action = 0;
            match conn_resp {
                Ok(body) => {
                    let character = body.into_inner();
                    user_characters.characters.push(super::Character {
                        id: character.id,
                        name: character.name.clone(),
                        position: None,
                    });
                    app_state.set(AppState::CharSelectionMenu).unwrap();
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
}
