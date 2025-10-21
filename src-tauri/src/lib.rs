use tauri::Manager;

pub mod fits;
mod renderer;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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

            // Upload FITS data to GPU
            {
                let mut r = renderer.lock().unwrap();
                r.load_fits_data(fits_img.data, fits_img.width, fits_img.height)?;
                println!("✅ FITS data uploaded to GPU");

                // Create the render pipeline with the actual surface format
                r.create_pipeline(surface_format)?;
                println!(
                    "✅ Render pipeline created with format: {:?}",
                    surface_format
                );
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
