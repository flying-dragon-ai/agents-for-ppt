pub mod export;
pub mod finalize;
pub mod ingest;
pub mod jobs;
pub mod project;

#[tauri::command]
pub fn cmd_hello_world() -> String {
    "Hello from Agents for PPT!".to_string()
}
