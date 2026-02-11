mod commands;
mod events;
mod state;

use state::AppState;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("projects")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new(workspace_root());

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::cmd_hello_world,
            commands::jobs::cmd_run_pipeline,
            commands::jobs::cmd_get_job_status,
            commands::jobs::cmd_cancel_job,
            commands::project::cmd_init_project,
            commands::project::cmd_validate_project,
            commands::project::cmd_get_project_info,
            commands::project::cmd_list_projects,
            commands::project::cmd_delete_project,
            commands::ingest::cmd_pdf_to_md,
            commands::ingest::cmd_web_to_md,
            commands::ingest::cmd_batch_pdf_to_md,
            commands::finalize::cmd_finalize_project,
            commands::export::cmd_export_pptx,
            commands::export::cmd_check_pptx_backends,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
