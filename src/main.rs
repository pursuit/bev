use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<ButtonMaterials>()
        .add_startup_system(setup_button.system())
        .add_system(button_system.system())
        .add_startup_system(setup_fps.system())
        .add_system(fps_update_system.system())
        .run();
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
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Connecting".to_string();
                *material = button_materials.pressed.clone();
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

fn setup_button(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    commands
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
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
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
            });
        });
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
struct FpsText;

fn setup_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());
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

