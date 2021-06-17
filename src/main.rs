use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use bev::pursuit::api::mortalkin::game_client::GameClient;
use bev::pursuit::api::mortalkin::user_client::UserClient;
use bev::system;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use bevy_tilemap::prelude::*;

use futures::executor::block_on;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut grpc_client_login = create_grpc_client().await;
    let (request_sender, request_receiver) = mpsc::channel();
    let (response_sender, response_receiver) = mpsc::channel();
    thread::spawn(move || loop {
        let payload = request_receiver.recv().unwrap();
        let response = grpc_client_login.login(payload);
        let resp = block_on(response);
        response_sender.send(resp).unwrap();
    });

    let mut grpc_client_create_char = create_grpc_client().await;
    let (create_char_request_sender, create_char_request_receiver) = mpsc::channel();
    let (create_char_response_sender, create_char_response_receiver) = mpsc::channel();
    thread::spawn(move || loop {
        let payload = create_char_request_receiver.recv().unwrap();
        let response = grpc_client_create_char.create_character(payload);
        let resp = block_on(response);
        create_char_response_sender.send(resp).unwrap();
    });

    let mut grpc_client_play = create_grpc_client_game().await;
    let (play_request_sender, mut play_request_receiver) = futures::channel::mpsc::unbounded();
    let (_play_response_sender, play_response_receiver) = futures::channel::mpsc::unbounded();

    let outbound = async_stream::stream! {
        let mut interval = time::interval(Duration::from_secs(1));

        while let _ = interval.tick().await {
            let next_payload = play_request_receiver.try_next();
            if let Ok(Some(payload)) = next_payload {
                yield payload;
            }
        }
    };

    thread::spawn(move || {
        let response = block_on(grpc_client_play.play(outbound)).unwrap();
        let mut inbound = response.into_inner();

        while let Some(game_notif) = block_on(inbound.message()).unwrap() {
            // play_response_sender.unbounded_send(game_notif).unwrap();
            println!("{:?}", game_notif);
        }
    });

    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapDefaultPlugins)
        .add_state(system::AppState::MainMenu)
        .init_resource::<system::ButtonMaterials>()
        .init_resource::<system::TileSpriteHandles>()
        .init_resource::<system::GameMap>()
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup_fps.system())
        .add_startup_system(system::setup_camera.system())
        .add_startup_system(system::setup_tile.system())
        .add_system(fps_update_system.system())
        .insert_resource(system::RequestSender {
            tx: Mutex::new(play_request_sender),
        })
        .insert_resource(system::ResponseReceiver {
            rx: Mutex::new(play_response_receiver),
        })
        .insert_resource(system::login::LoginAction::new())
        .insert_resource(system::login::LoginRequestSender {
            tx: Mutex::new(request_sender),
        })
        .insert_resource(system::login::LoginResponseReceiver {
            rx: Mutex::new(response_receiver),
        })
        .insert_resource(system::char_creation::Action::new())
        .insert_resource(system::char_creation::RequestSender {
            tx: Mutex::new(create_char_request_sender),
        })
        .insert_resource(system::char_creation::ResponseReceiver {
            rx: Mutex::new(create_char_response_receiver),
        })
        .add_system_set(
            SystemSet::on_enter(system::AppState::MainMenu)
                .with_system(system::login::setup_login_form.system()),
        )
        .add_system_set(
            SystemSet::on_update(system::AppState::MainMenu)
                .with_system(system::login::login_button_system.system())
                .with_system(system::login::login_input_event_system.system())
                .with_system(system::login::login_system.system()),
        )
        .add_system_set(
            SystemSet::on_exit(system::AppState::MainMenu)
                .with_system(system::login::cleanup_login_form.system()),
        )
        .add_system_set(
            SystemSet::on_enter(system::AppState::CharSelectionMenu)
                .with_system(system::char_selection::setup_system.system()),
        )
        .add_system_set(
            SystemSet::on_update(system::AppState::CharSelectionMenu)
                .with_system(system::char_selection::create_button_system.system())
                .with_system(system::char_selection::play_button_system.system()),
        )
        .add_system_set(
            SystemSet::on_exit(system::AppState::CharSelectionMenu)
                .with_system(system::char_selection::cleanup.system()),
        )
        .add_system_set(
            SystemSet::on_enter(system::AppState::CharCreationMenu)
                .with_system(system::char_creation::setup_create_form.system()),
        )
        .add_system_set(
            SystemSet::on_update(system::AppState::CharCreationMenu)
                .with_system(system::char_creation::create_button_system.system())
                .with_system(system::char_creation::input_event_system.system())
                .with_system(system::char_creation::submit_system.system()),
        )
        .add_system_set(
            SystemSet::on_exit(system::AppState::CharCreationMenu)
                .with_system(system::char_creation::cleanup.system()),
        )
        .add_system_set(
            SystemSet::on_update(system::AppState::Field)
                .with_system(system::field::load.system())
                .with_system(system::field::build.system())
                .with_system(system::character_movement.system()),
        )
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

async fn create_grpc_client_game() -> GameClient<tonic::transport::Channel> {
    let channel = tonic::transport::Channel::from_static("http://[::1]:5004")
        .connect()
        .await
        .expect("Can't create a channel");

    GameClient::new(channel)
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
