use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

pub mod fits;
mod renderer;

// State to hold the renderer
struct AppState {
    renderer: Arc<Mutex<renderer::FitsRenderer>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn update_view(state: State<AppState>, zoom: f32, pan_x: f32, pan_y: f32) {
    let renderer = state.renderer.lock().unwrap();
    renderer.update_view(zoom, pan_x, pan_y);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load FITS file first
    let fits_path = "C:/Users/chris/Downloads/RemoteAstrophotography-com-NGC300-LRGB/NGC300-L.fit";
    let fits_img = fits::load_fits_f32(fits_path).expect("Failed to load FITS file");
    println!("📷 Loaded FITS: {}x{}", fits_img.width, fits_img.height);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // Create the main UI window
            let main_window = app.get_webview_window("main").expect("main window");

            // Initialize WGPU renderer on the main window (returns renderer and surface format)
            let (renderer, surface_format) = renderer::init_renderer_for_window(&main_window)?;

            // Get window size for aspect ratio
            let window_size = main_window.inner_size()?;

            // Upload FITS data to GPU
            {
                let mut r = renderer.lock().unwrap();
                r.load_fits_data(fits_img.data, fits_img.width, fits_img.height)?;
                println!("✅ FITS data uploaded to GPU");

                // Create the render pipeline with the actual surface format and viewport size
                r.create_pipeline(surface_format, window_size.width, window_size.height)?;
                println!(
                    "✅ Render pipeline created with format: {:?}",
                    surface_format
                );
            }

            // Store renderer in app state
            app.manage(AppState { renderer });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, update_view])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
