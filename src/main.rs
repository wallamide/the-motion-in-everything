// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use the_motion_in_everything::GamePlugin;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}
