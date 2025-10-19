// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rapidfits_lib::fits::load_fits_f32;

fn main() {
    rapidfits_lib::run();
}
