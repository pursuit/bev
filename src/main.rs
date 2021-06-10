use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use bev::pursuit::api::mortalkin::{user_client::UserClient, LoginPayload, LoginResponse};
use futures::executor::block_on;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    CharMenu,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut grpc_client = create_grpc_client().await;

    let (request_sender, request_receiver) = mpsc::channel();
    let (response_sender, response_receiver) = mpsc::channel();
    thread::spawn(move || loop {
        let payload = request_receiver.recv().unwrap();
        let response = grpc_client.login(payload);
        let resp = block_on(response);
        response_sender.send(resp).unwrap();
    });

    App::build()
        .add_plugins(DefaultPlugins)
        .add_state(AppState::MainMenu)
        .init_resource::<ButtonMaterials>()
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup_fps.system())
        .add_system(fps_update_system.system())
        .insert_resource(LoginAction::new())
        .insert_resource(LoginRequestSender {
            tx: Mutex::new(request_sender),
        })
        .insert_resource(LoginResponseReceiver {
            rx: Mutex::new(response_receiver),
        })
        .add_system_set(
            SystemSet::on_enter(AppState::MainMenu).with_system(setup_login_form.system().system()),
        )
        .add_system_set(
            SystemSet::on_update(AppState::MainMenu)
                .with_system(login_button_system.system())
                .with_system(login_input_event_system.system())
                .with_system(login_system.system()),
        )
        .run();

    Ok(())
}

struct LoginRequestSender {
    tx: Mutex<mpsc::Sender<LoginPayload>>,
}

struct LoginResponseReceiver {
    rx: Mutex<mpsc::Receiver<Result<tonic::Response<LoginResponse>, tonic::Status>>>,
}

async fn create_grpc_client() -> UserClient<tonic::transport::Channel> {
    let channel = tonic::transport::Channel::from_static("http://[::1]:5004")
        .connect()
        .await
        .expect("Can't create a channel");

    UserClient::new(channel)
}

struct ButtonMaterials {
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

fn login_button_system(
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

// A unit struct to help identify the FPS UI component, since there may be many Text components
struct FpsText;

fn setup_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Rich text with multiple sections
    commands
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
                        value: "FPS: ".to_string(),
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
        .insert(FpsText);
}

fn fps_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}

struct UsernameText;
struct PasswordText;
struct LoginButtonText;
struct LoginAction {
    action: u32,
}

impl LoginAction {
    pub fn new() -> Self {
        Self { action: 0 }
    }
}

fn setup_login_form(
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
                        .insert(LoginButtonText);
                });
        });
}

fn login_input_event_system(
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

fn login_system(
    mut action: ResMut<LoginAction>,
    mut app_state: ResMut<State<AppState>>,
    login_response_receiver: ResMut<LoginResponseReceiver>,
) {
    if action.action != 2 {
        return;
    }

    let resp = login_response_receiver.rx.lock().unwrap().try_recv();
    match resp {
        Ok(body_resp) => {
            action.action = 0;
            match body_resp {
                Ok(_) => {
                    app_state.set(AppState::CharMenu).unwrap();
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
}
