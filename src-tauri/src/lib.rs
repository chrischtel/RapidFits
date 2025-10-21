use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

pub mod fits;
mod renderer;

// State to hold the renderer and image data
struct AppState {
    renderer: Arc<Mutex<renderer::FitsRenderer>>,
    stats: Arc<fits::ImageStats>,
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

#[tauri::command]
fn update_stretch(state: State<AppState>, min: f32, max: f32) {
    let renderer = state.renderer.lock().unwrap();
    renderer.update_stretch(min, max);
}

#[tauri::command]
fn get_image_stats(state: State<AppState>) -> fits::ImageStats {
    (*state.stats).clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load FITS file first
    let fits_path = "C:/Users/chris/Downloads/RemoteAstrophotography-com-NGC300-LRGB/NGC300-L.fit";
    let fits_img = fits::load_fits_f32(fits_path).expect("Failed to load FITS file");
    println!("📷 Loaded FITS: {}x{}", fits_img.width, fits_img.height);

    // Print statistics
    println!("📊 Statistics:");
    println!(
        "   Min: {:.2}, Max: {:.2}",
        fits_img.stats.min, fits_img.stats.max
    );
    println!(
        "   Mean: {:.2}, StdDev: {:.2}",
        fits_img.stats.mean, fits_img.stats.stddev
    );
    println!("   Median: {:.2}", fits_img.stats.median);

    // Calculate auto-stretch (0.5% to 99.5% percentile)
    let (stretch_min, stretch_max) =
        fits::calculate_auto_stretch(&fits_img.stats, &fits_img.data, 0.5, 99.5);
    println!("🎨 Auto-stretch: {:.2} to {:.2}", stretch_min, stretch_max);

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

                // Apply auto-stretch
                r.update_stretch(stretch_min, stretch_max);
                println!(
                    "✅ Applied auto-stretch: {:.2} to {:.2}",
                    stretch_min, stretch_max
                );
            }

            // Store renderer and stats in app state
            app.manage(AppState {
                renderer,
                stats: Arc::new(fits_img.stats),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            update_view,
            update_stretch,
            get_image_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
