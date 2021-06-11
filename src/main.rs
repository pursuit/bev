use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use bev::pursuit::api::mortalkin::user_client::UserClient;
use bev::system;

use futures::executor::block_on;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut grpc_client_login = create_grpc_client().await;
    let mut grpc_client_create_char = create_grpc_client().await;

    let (request_sender, request_receiver) = mpsc::channel();
    let (response_sender, response_receiver) = mpsc::channel();
    thread::spawn(move || loop {
        let payload = request_receiver.recv().unwrap();
        let response = grpc_client_login.login(payload);
        let resp = block_on(response);
        response_sender.send(resp).unwrap();
    });

    let (create_char_request_sender, create_char_request_receiver) = mpsc::channel();
    let (create_char_response_sender, create_char_response_receiver) = mpsc::channel();
    thread::spawn(move || loop {
        let payload = create_char_request_receiver.recv().unwrap();
        let response = grpc_client_create_char.create_character(payload);
        let resp = block_on(response);
        create_char_response_sender.send(resp).unwrap();
    });

    App::build()
        .add_plugins(DefaultPlugins)
        .add_state(system::AppState::MainMenu)
        .init_resource::<system::ButtonMaterials>()
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup_fps.system())
        .add_system(fps_update_system.system())
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
                .with_system(system::char_selection::create_button_system.system()),
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
