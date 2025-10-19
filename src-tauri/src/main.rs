// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rapidfits_lib::fits::load_fits_f32;

fn main() {
    let img = load_fits_f32(
        "C:/Users/chris/Downloads/RemoteAstrophotography-com-NGC300-LRGB/NGC300-L.fit",
    )
    .unwrap();

    println!("{:?}:{:?}", img.height, img.width);
    rapidfits_lib::run();
}
