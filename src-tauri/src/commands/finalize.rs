// Tauri 命令：SVG 后处理

use pptm_pipeline::steps::finalize::{finalize_project, FinalizeOptions};
use std::path::PathBuf;

#[tauri::command]
pub async fn cmd_finalize_project(
    project_path: String,
    embed_icons: Option<bool>,
    crop_images: Option<bool>,
    fix_aspect: Option<bool>,
    embed_images: Option<bool>,
    flatten_text: Option<bool>,
    fix_rounded: Option<bool>,
) -> Result<(), String> {
    let project_path = PathBuf::from(project_path);

    let options = FinalizeOptions {
        embed_icons: embed_icons.unwrap_or(true),
        crop_images: crop_images.unwrap_or(true),
        fix_aspect: fix_aspect.unwrap_or(true),
        embed_images: embed_images.unwrap_or(true),
        flatten_text: flatten_text.unwrap_or(true),
        fix_rounded: fix_rounded.unwrap_or(true),
    };

    finalize_project(&project_path, &options).map_err(|e| e.to_string())
}
