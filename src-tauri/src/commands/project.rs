use pptm_domain::{ProjectInfo, ValidationResult};
use pptm_pipeline::steps::project_manager::{
    delete_project, get_project_info_detailed, init_project, list_projects, validate_project,
};
use serde::{Deserialize, Serialize};

/// 初始化项目请求。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitProjectRequest {
    pub name: String,
    pub format: String,
    pub base_dir: Option<String>,
}

/// 初始化项目响应。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitProjectResponse {
    pub project_path: String,
}

#[tauri::command]
pub fn cmd_init_project(request: InitProjectRequest) -> Result<InitProjectResponse, String> {
    let base_dir = request.base_dir.as_deref();
    let project_path = init_project(&request.name, &request.format, base_dir)
        .map_err(|error| error.to_string())?;

    Ok(InitProjectResponse { project_path })
}

#[tauri::command]
pub fn cmd_validate_project(project_path: String) -> Result<ValidationResult, String> {
    validate_project(project_path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn cmd_get_project_info(project_path: String) -> Result<ProjectInfo, String> {
    get_project_info_detailed(project_path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn cmd_list_projects(base_dir: Option<String>) -> Result<Vec<String>, String> {
    list_projects(base_dir.as_deref()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn cmd_delete_project(project_path: String) -> Result<(), String> {
    delete_project(project_path).map_err(|error| error.to_string())
}
