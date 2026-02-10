#[tauri::command]
pub fn cmd_hello_world() -> String {
    "Hello from Agents for PPT!".to_string()
}
