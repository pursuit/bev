use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use bev::pursuit::api::mortalkin::{user_client::UserClient, LoginPayload};
use futures::executor::block_on;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_client = create_grpc_client().await;

    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<ButtonMaterials>()
        .insert_resource(grpc_client)
        .insert_resource(LoginAction::new())
        .add_system(button_system.system())
        .add_startup_system(setup_fps.system())
        .add_system(fps_update_system.system())
        .add_startup_system(setup_form.system())
        .add_system(input_event_system.system())
        .run();

    Ok(())
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

fn button_system(
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
    mut grpc_conn: ResMut<UserClient<tonic::transport::Channel>>,
) {
    let username = user_query.single_mut().unwrap().sections[1].value.clone();
    let password = password_query.single_mut().unwrap().sections[1]
        .value
        .clone();

    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Connecting".to_string();
                *material = button_materials.pressed.clone();

                let response = grpc_conn.login(LoginPayload {
                    username: username.clone(),
                    password: password.clone().as_bytes().to_vec(),
                });

                let resp = block_on(response);
                println!("RESPONSE={:?}", resp);
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

fn setup_form(
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

fn input_event_system(
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
