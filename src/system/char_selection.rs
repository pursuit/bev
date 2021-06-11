use bevy::prelude::*;

use super::AppState;
use super::ButtonMaterials;
use super::UserCharacters;

pub struct CreateButtonText;
pub struct PlayButtonText;
pub struct CleanupEntity;

pub fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    user_characters: Res<UserCharacters>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    button_materials: Res<ButtonMaterials>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    for n in 0..3 {
        if user_characters.characters.len() <= n {
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
                .insert(CleanupEntity)
                .with_children(|parent| {
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
                        .insert(CleanupEntity)
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
                                .insert(CleanupEntity)
                                .insert(CreateButtonText);
                        });
                });
        } else {
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
                .insert(CleanupEntity)
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
                                sections: vec![TextSection {
                                    value: user_characters.characters[n].name.clone(),
                                    style: TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.otf"),
                                        font_size: 60.0,
                                        color: Color::WHITE,
                                    },
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(CleanupEntity);
                });
        }
    }
}

pub fn create_button_system(
    button_materials: Res<ButtonMaterials>,
    mut app_state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text, With<CreateButtonText>>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                app_state.set(AppState::CharCreationMenu).unwrap();
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

pub fn cleanup(mut commands: Commands, q: Query<Entity, With<CleanupEntity>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}
