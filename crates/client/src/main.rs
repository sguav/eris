// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tauri_context() {
        // This verifies that the tauri.conf.json and resources (like icons) are valid
        let _context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    }
}
