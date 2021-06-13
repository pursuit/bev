use std::sync::mpsc;
use std::sync::Mutex;

use super::AppState;
use super::ButtonMaterials;
use super::Character;
use super::Token;
use super::UserCharacters;
use crate::pursuit::api::mortalkin::{LoginPayload, LoginResponse};

use bevy::prelude::*;

pub struct LoginRequestSender {
    pub tx: Mutex<mpsc::Sender<LoginPayload>>,
}

pub struct LoginResponseReceiver {
    pub rx: Mutex<mpsc::Receiver<Result<tonic::Response<LoginResponse>, tonic::Status>>>,
}

pub struct UsernameText;
pub struct PasswordText;

pub struct LoginButtonText;
pub struct LoginFormUI;
pub struct LoginAction {
    action: u32,
}

impl LoginAction {
    pub fn new() -> Self {
        Self { action: 0 }
    }
}

pub fn setup_login_form(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    button_materials: Res<ButtonMaterials>,
) {
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
        .insert(LoginFormUI)
        .with_children(|parent| {
            // left vertical fill (border)
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
                                value: "Username: ".to_string(),
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
                .insert(LoginFormUI)
                .insert(UsernameText);

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
                                value: "Password: ".to_string(),
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
                .insert(LoginFormUI)
                .insert(PasswordText);

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
                .insert(LoginFormUI)
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
                        .insert(LoginFormUI)
                        .insert(LoginButtonText);
                });
        });
}

pub fn cleanup_login_form(mut commands: Commands, q: Query<Entity, With<LoginFormUI>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

pub fn login_button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<
        &mut Text,
        (
            With<LoginButtonText>,
            Without<UsernameText>,
            Without<PasswordText>,
        ),
    >,
    mut user_query: Query<&mut Text, (With<UsernameText>, Without<PasswordText>)>,
    mut password_query: Query<&mut Text, With<PasswordText>>,
    login_request_sender: ResMut<LoginRequestSender>,
    mut action: ResMut<LoginAction>,
) {
    if action.action == 2 {
        return;
    }

    let username = user_query.single_mut().unwrap().sections[1].value.clone();
    let password = password_query.single_mut().unwrap().sections[1]
        .value
        .clone();

    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                action.action = 2;
                text.sections[0].value = "Connecting".to_string();
                *material = button_materials.pressed.clone();

                login_request_sender
                    .tx
                    .lock()
                    .unwrap()
                    .send(LoginPayload {
                        username: username.clone(),
                        password: password.clone().as_bytes().to_vec(),
                    })
                    .unwrap();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                text.sections[0].value = "Login".to_string();
                *material = button_materials.normal.clone();
            }
        }
    }
}

pub fn login_input_event_system(
    mut action: ResMut<LoginAction>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut username_query: Query<&mut Text, (With<UsernameText>, Without<PasswordText>)>,
    mut password_query: Query<&mut Text, With<PasswordText>>,
) {
    if action.action == 2 {
        return;
    }

    for event in char_input_events.iter() {
        if event.char == ' ' {
            action.action ^= 1;
        } else if action.action == 0 {
            let username = &mut username_query.single_mut().unwrap().sections[1];
            if event.char == '\x08' {
                username.value.pop();
            } else {
                username.value.push(event.char);
            }
        } else {
            let password = &mut password_query.single_mut().unwrap().sections[1];
            if event.char == '\x08' {
                password.value.pop();
            } else {
                password.value.push(event.char);
            }
        }
    }
}

pub fn login_system(
    mut commands: Commands,
    mut action: ResMut<LoginAction>,
    mut app_state: ResMut<State<AppState>>,
    login_response_receiver: ResMut<LoginResponseReceiver>,
) {
    if action.action != 2 {
        return;
    }

    let resp = login_response_receiver.rx.lock().unwrap().try_recv();
    match resp {
        Ok(conn_resp) => {
            action.action = 0;
            match conn_resp {
                Ok(body) => {
                    let inner = body.into_inner();
                    commands.insert_resource(Token { token: inner.token });

                    let characters = inner
                        .characters
                        .iter()
                        .map(|character| Character {
                            id: character.id,
                            name: character.name.clone(),
                            position: None,
                        })
                        .collect();

                    commands.insert_resource(UserCharacters {
                        characters: characters,
                    });
                    app_state.set(AppState::CharSelectionMenu).unwrap();
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
}
