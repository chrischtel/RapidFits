use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

pub mod fits;
mod renderer;

// State to hold the renderer and image data
struct AppState {
    renderer: Arc<Mutex<renderer::FitsRenderer>>,
    stats: Arc<Mutex<fits::ImageStats>>,
    surface_format: Arc<Mutex<wgpu::TextureFormat>>,
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
    (*state.stats.lock().unwrap()).clone()
}

#[tauri::command]
async fn open_single_fits_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<fits::ImageStats, String> {
    // Load FITS file
    let fits_img = fits::load_fits_f32(&path).map_err(|e| format!("Failed to load FITS: {}", e))?;

    println!("Loaded FITS: {}x{}", fits_img.width, fits_img.height);
    println!("Statistics:");
    println!(
        "   Min: {:.2}, Max: {:.2}",
        fits_img.stats.min, fits_img.stats.max
    );
    println!(
        "   Mean: {:.2}, StdDev: {:.2}",
        fits_img.stats.mean, fits_img.stats.stddev
    );
    println!("   Median: {:.2}", fits_img.stats.median);

    // Calculate auto-stretch
    let (stretch_min, stretch_max) =
        fits::calculate_auto_stretch(&fits_img.stats, &fits_img.data, 0.5, 99.5);
    println!("Auto-stretch: {:.2} to {:.2}", stretch_min, stretch_max);

    // Update renderer with new data
    {
        let mut renderer = state.renderer.lock().unwrap();
        let surface_format = *state.surface_format.lock().unwrap();

        // Upload new FITS data to GPU
        renderer
            .load_fits_data(fits_img.data, fits_img.width, fits_img.height)
            .map_err(|e| format!("Failed to upload to GPU: {}", e))?;

        // Recreate pipeline with new dimensions (assume window size hasn't changed)
        // Note: In a real app, you'd get the actual window size here
        renderer
            .create_pipeline(surface_format, 1200, 800)
            .map_err(|e| format!("Failed to create pipeline: {}", e))?;

        // Apply auto-stretch
        renderer.update_stretch(stretch_min, stretch_max);

        println!("FITS data uploaded to GPU and pipeline updated");
    }

    // Update stats in state
    let new_stats = fits_img.stats.clone();
    *state.stats.lock().unwrap() = fits_img.stats;

    Ok(new_stats)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // Create the main UI window
            let main_window = app.get_webview_window("main").expect("main window");

            // Initialize WGPU renderer on the main window (returns renderer and surface format)
            let (renderer, surface_format) = renderer::init_renderer_for_window(&main_window)?;

            println!("WGPU renderer initialized, waiting for FITS file to be loaded");

            // Create placeholder/empty stats
            let placeholder_stats = fits::ImageStats {
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                stddev: 0.0,
                median: 0.0,
                histogram: vec![0; 256],
            };

            // Store renderer and stats in app state (no image loaded yet)
            app.manage(AppState {
                renderer,
                stats: Arc::new(Mutex::new(placeholder_stats)),
                surface_format: Arc::new(Mutex::new(surface_format)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            update_view,
            update_stretch,
            get_image_stats,
            open_single_fits_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
