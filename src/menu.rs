use crate::{file_system_interaction::asset_loading::FontAssets, GameState};
use bevy::prelude::*;

#[derive(Component)]
struct MainMenuLabel;

#[derive(Component)]
struct PlayBtn;

#[derive(Resource)]
struct MainMenuColors {
    btn_bg_base: Color,
    btn_bg_hovered: Color,
    btn_txt: Color,
    menu_bg: Color,
    title_txt_color: Color,
}

impl Default for MainMenuColors {
    fn default() -> Self {
        MainMenuColors {
            btn_bg_base: Color::rgb(0.15, 0.15, 0.15),
            btn_bg_hovered: Color::rgb(0.25, 0.25, 0.25),
            btn_txt: Color::DARK_GREEN,
            menu_bg: Color::rgb(0.81, 0.12, 0.988),
            title_txt_color: Color::BLACK,
        }
    }
}

/// This plugin is responsible for the game menu
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited.
pub fn menu_plugin(app: &mut App) {
    app.add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
        // .add_system(button_actions.in_set(OnUpdate(GameState::Menu)))
        .add_system(cleanup_menu.in_schedule(OnExit(GameState::Menu)));
}

// Modified from https://github.com/rparrett/undefended/blob/main/src/main_menu.rs
fn setup_menu(mut commands: Commands, fonts: Res<FontAssets>, colors: Res<MainMenuColors>) {
    let btn_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(45.0)),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let btn_txt_style = TextStyle {
        font: fonts.grasshopper.clone(),
        font_size: 30.0,
        color: colors.btn_txt.into(),
    };
    let title_txt_style = TextStyle {
        font: fonts.caprice.clone(),
        font_size: 80.0,
        color: colors.title_txt_color.into(),
    };

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                background_color: colors.menu_bg.into(),
                ..default()
            },
            MainMenuLabel,
        ))
        .id();

    let title = commands
        .spawn(
            TextBundle::from_section("The Motion in Everything", title_txt_style).with_style(
                Style {
                    margin: UiRect {
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .id();

    let play_btn = commands
        .spawn((
            ButtonBundle {
                style: btn_style.clone(),
                background_color: colors.btn_bg_base.into(),
                ..default()
            },
            // Focusable::default(),
            MenuAction::Play,
            PlayBtn,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("PLAY", btn_txt_style.clone()));
        })
        .id();
    commands.entity(container).push_children(&[title, play_btn]);
}

#[derive(Component, Debug)]
enum MenuAction {
    Play,
}

// fn button_actions(
//     // mut events: EventReader<NavEvent>,
//     btn_colors: Res<MainMenuColors>,
//     mut interactions: Query<
//         (&Interaction, &mut BackgroundColor, &MenuAction),
//         (Changed<Interaction>, With<MenuAction>),
//     >,
//     mut next_state: ResMut<NextState<GameState>>,
// ) {
//     for (interaction, mut color, btn) in &mut interactions {
//         match *interaction {
//             Interaction::Clicked => match btn {
//                 MenuAction::Play => next_state.set(GameState::Playing),
//                 // e => error!("Unable to click on {e:?} on main menu"),
//             },
//             Interaction::Hovered => *color = btn_colors.btn_bg_hovered.into(),
//             Interaction::None => *color = btn_colors.btn_bg_base.into(),
//         }
//     }
// }

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MainMenuLabel>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
