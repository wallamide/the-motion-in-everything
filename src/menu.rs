use crate::GameState;
use bevy::prelude::*;

/// Label for the main menu container
#[derive(Component)]
struct MainMenuContainer;

#[derive(Component)]
struct BtnContainer;

#[derive(Component)]
struct LinkContainer;

/// Label for the play button
#[derive(Component)]
struct PlayBtn;

#[derive(Component)]
struct LoadBtn;

#[derive(Component)]
struct SettingsBtn;

#[derive(Component)]
struct QuitBtn;

#[derive(Resource)]
struct MainMenuColors {
    btn_container_bg: Color,
    btn_bg_base: Color,
    btn_bg_hovered: Color,
    btn_txt: Color,
    menu_bg: Color,
    title_txt_color: Color,
}

impl Default for MainMenuColors {
    fn default() -> Self {
        MainMenuColors {
            btn_container_bg: Color::GRAY,
            btn_bg_base: Color::rgb(0.406, 0.375, 0.43297),
            btn_bg_hovered: Color::rgb(0.613, 0.508, 0.688),
            btn_txt: Color::BLACK,
            menu_bg: Color::rgb(0.37, 0.305, 0.54),
            title_txt_color: Color::BLACK,
        }
    }
}

// ToDo: choose fonts
#[derive(Resource, Clone)]
pub struct FontAssets {
    pub caprice: Handle<Font>,
    pub grasshopper: Handle<Font>,
}

/// This plugin is responsible for the game menu
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited.
pub fn menu_plugin(app: &mut App) {
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();

    app.insert_resource(FontAssets {
        caprice: asset_server.load("fonts/caprice.ttf"),
        grasshopper: asset_server.load("fonts/grasshopper.ttf"),
    })
    .insert_resource(MainMenuColors::default())
    .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
    // .add_system(setup_settings.in_schedule(OnEnter(GameState::Settings)))
    .add_system(button_actions.in_set(OnUpdate(GameState::Menu)))
    .add_system(cleanup_menu.in_schedule(OnExit(GameState::Menu)));
}

// Modified from https://github.com/rparrett/undefended/blob/main/src/main_menu.rs
fn setup_menu(mut commands: Commands, fonts: Res<FontAssets>, colors: Res<MainMenuColors>) {
    let main_menu_style = Style {
        margin: UiRect::all(Val::Auto),
        size: Size::all(Val::Percent(100.0)),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Percent(5.0)),
        ..default()
    };

    let btn_container_style = Style {
        size: Size::new(Val::Px(300.0), Val::Percent(45.0)),
        flex_direction: FlexDirection::Column,
        align_self: AlignSelf::FlexStart,
        justify_content: JustifyContent::Center,
        position: UiRect::top(Val::Percent(45.0)),
        ..default()
    };

    let link_container_style = Style {
        size: Size::new(Val::Px(200.0), Val::Percent(35.0)),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_self: AlignSelf::FlexEnd,
        position: UiRect::bottom(Val::Percent(25.0)),
        ..default()
    };

    let btn_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(45.0)),
        margin: UiRect::all(Val::Percent(2.0)),
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Percent(2.0)),
        ..default()
    };

    let title_style = Style {
        margin: UiRect::all(Val::Percent(1.25)),
        position_type: PositionType::Absolute,
        flex_wrap: FlexWrap::Wrap,
        align_self: AlignSelf::Center,
        ..default()
    };

    let btn_txt_style = TextStyle {
        font: fonts.grasshopper.clone(),
        font_size: 35.0,
        color: colors.btn_txt,
    };

    let btn_txt_spacing = Style {
        margin: UiRect::left(Val::Percent(5.0)),
        ..default()
    };

    let title_txt_style = TextStyle {
        font: fonts.caprice.clone(),
        font_size: 60.0,
        color: colors.title_txt_color,
    };

    let menu_container = commands
        .spawn((
            NodeBundle {
                style: main_menu_style.into(),
                background_color: colors.menu_bg.into(),
                ..default()
            },
            MainMenuContainer,
        ))
        .id();

    let title = commands
        .spawn(
            TextBundle::from_section("The Motion in Everything", title_txt_style).with_style(
                title_style.into(),
            ),
        )
        .id();
    
    let btn_container = commands
        .spawn((
            NodeBundle{
                style: btn_container_style,
                background_color: colors.btn_container_bg.into(),
                ..default()
            },
            BtnContainer,
        ))
        .id();

    let play_btn = commands
        .spawn((
            ButtonBundle {
                style: btn_style.clone(),
                ..default()
            },
            // Focusable::default(), //ToDo: integrate bevy_ui_navigation
            MenuAction::Play,
            PlayBtn,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Start Game", btn_txt_style.clone()).with_style(btn_txt_spacing.clone())
            );
        })
        .id();

    let load_btn = commands
        .spawn((
            ButtonBundle {
                style: btn_style.clone(),
                ..default()
            },
            MenuAction::Load,
            LoadBtn,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Load Game", btn_txt_style.clone()).with_style(btn_txt_spacing.clone()));
        })
        .id();

    let settings_btn = commands
        .spawn((
            ButtonBundle {
                style: btn_style.clone(),
                ..default()
           },
            MenuAction::Settings,
            SettingsBtn,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Settings", btn_txt_style.clone()).with_style(btn_txt_spacing.clone()));
        })
        .id();

    let quit_btn = commands
        .spawn((
            ButtonBundle{
                style: btn_style.clone(),
                ..default()
            },
            MenuAction::Settings,
            QuitBtn,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Quit", btn_txt_style).with_style(btn_txt_spacing));
        })
        .id();

    let link_container = commands
        .spawn((
            NodeBundle{
                style: link_container_style.clone(),
                background_color: colors.btn_container_bg.into(),
                ..default()
            },
            LinkContainer,
        ))
        .id();

    commands.entity(btn_container).push_children(&[play_btn, load_btn, settings_btn, quit_btn]);

    commands.entity(menu_container).push_children(&[title, btn_container, link_container]);
}

// fn setup_settings(mut commands: Commands, fonts: Res<FontAssets>, colors: Res<MainMenuColors>){
    
// }

#[derive(Component, Debug)]
pub enum MenuAction {
    Play,
    Load,
    Settings,
    Quit,
}

fn button_actions(
    // mut events: EventReader<NavEvent>,
    ui_colors: Res<MainMenuColors>, // 
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor, &MenuAction),
        (Changed<Interaction>, With<MenuAction>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, btn) in &mut interactions {
        match *interaction {
            Interaction::Clicked => match btn {
                MenuAction::Play => next_state.set(GameState::Playing),
                MenuAction::Load => next_state.set(GameState::LoadSaves),
                MenuAction::Settings => next_state.set(GameState::Settings),
                MenuAction::Quit => next_state.set(GameState::Quit),
                // e => error!("Unable to click on {e:?} on main menu"),
            },
            Interaction::Hovered => *color = ui_colors.btn_bg_hovered.into(),
            Interaction::None => *color = ui_colors.btn_bg_base.into(),
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MainMenuContainer>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
