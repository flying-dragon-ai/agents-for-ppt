use anyhow::{Context, Result};
use chrono::Local;
use pptm_domain::{
    find_all_projects, get_project_info, normalize_canvas_format, validate_project_structure,
    ProjectInfo, ValidationResult, CANVAS_FORMATS,
};
use std::fs;
use std::path::{Path, PathBuf};

/// 初始化新项目。
///
/// # Arguments
///
/// * `project_name` - 项目名称
/// * `canvas_format` - 画布格式 (ppt169, ppt43, wechat, 等)
/// * `base_dir` - 项目基础目录，默认为 `projects`
///
/// # Returns
///
/// 创建的项目路径
pub fn init_project(
    project_name: &str,
    canvas_format: &str,
    base_dir: Option<&str>,
) -> Result<String> {
    let base_path = PathBuf::from(base_dir.unwrap_or("projects"));

    let normalized_format = normalize_canvas_format(canvas_format);
    if !CANVAS_FORMATS.contains_key(&normalized_format) {
        let available: Vec<_> = CANVAS_FORMATS.keys().map(|key| key.as_str()).collect();
        anyhow::bail!(
            "不支持的画布格式: {} (可用: {}; 常用别名: xhs -> xiaohongshu)",
            canvas_format,
            available.join(", ")
        );
    }

    let date_str = Local::now().format("%Y%m%d").to_string();
    let project_dir_name = format!("{}_{}_{}", project_name, normalized_format, date_str);
    let project_path = base_path.join(&project_dir_name);

    if project_path.exists() {
        anyhow::bail!("项目目录已存在: {}", project_path.display());
    }

    fs::create_dir_all(&project_path)
        .context(format!("创建项目目录失败: {}", project_path.display()))?;

    fs::create_dir(project_path.join("svg_output")).context("创建 svg_output 目录失败")?;
    fs::create_dir(project_path.join("svg_final")).context("创建 svg_final 目录失败")?;
    fs::create_dir(project_path.join("images")).context("创建 images 目录失败")?;
    fs::create_dir(project_path.join("notes")).context("创建 notes 目录失败")?;
    fs::create_dir(project_path.join("templates")).context("创建 templates 目录失败")?;

    let canvas_info = CANVAS_FORMATS
        .get(&normalized_format)
        .expect("画布格式应在校验后存在");
    let readme_content = format!(
        "# {}\n\n\
         - 画布格式: {}\n\
         - 创建日期: {}\n\n\
         ## 目录\n\n\
         - `svg_output/`: 原始 SVG 输出\n\
         - `svg_final/`: 后处理后的 SVG\n\
         - `images/`: 图片资源\n\
         - `notes/`: 演讲备注\n\
         - `templates/`: 项目模板\n",
        project_name, normalized_format, date_str
    );

    fs::write(project_path.join("README.md"), readme_content).context("创建 README.md 失败")?;

    println!("项目目录已创建: {}", project_path.display());
    println!(
        "画布格式: {} ({})",
        canvas_info.name, canvas_info.dimensions
    );

    Ok(project_path.to_string_lossy().to_string())
}

/// 验证项目完整性。
pub fn validate_project<P: AsRef<Path>>(project_path: P) -> Result<ValidationResult> {
    Ok(validate_project_structure(project_path, false))
}

/// 获取项目信息。
pub fn get_project_info_detailed<P: AsRef<Path>>(project_path: P) -> Result<ProjectInfo> {
    Ok(get_project_info(project_path))
}

/// 列出所有项目。
pub fn list_projects(base_dir: Option<&str>) -> Result<Vec<String>> {
    let base_path = base_dir.unwrap_or("projects");
    let projects = find_all_projects(base_path);
    Ok(projects
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect())
}

/// 删除项目目录。
pub fn delete_project<P: AsRef<Path>>(project_path: P) -> Result<()> {
    let project_path = project_path.as_ref();

    if !project_path.exists() {
        anyhow::bail!("项目目录不存在: {}", project_path.display());
    }

    if !project_path.is_dir() {
        anyhow::bail!("项目路径不是目录: {}", project_path.display());
    }

    if !looks_like_project(project_path) {
        anyhow::bail!(
            "目标目录不像 PPT 项目，已拒绝删除: {}",
            project_path.display()
        );
    }

    fs::remove_dir_all(project_path)
        .context(format!("删除项目目录失败: {}", project_path.display()))?;

    Ok(())
}

fn looks_like_project(project_path: &Path) -> bool {
    let markers = ["svg_output", "svg_final", "images", "notes", "templates"];
    markers
        .iter()
        .filter(|name| project_path.join(name).exists())
        .count()
        >= 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_project() {
        let temp_dir = TempDir::new().expect("应能创建临时目录");
        let base_dir = temp_dir.path().to_str().expect("应能转换为字符串");

        let result = init_project("test_project", "ppt169", Some(base_dir));
        assert!(result.is_ok());

        let project_path = PathBuf::from(result.expect("应返回项目路径"));

        assert!(project_path.exists());
        assert!(project_path.join("svg_output").exists());
        assert!(project_path.join("svg_final").exists());
        assert!(project_path.join("images").exists());
        assert!(project_path.join("notes").exists());
        assert!(project_path.join("templates").exists());
        assert!(project_path.join("README.md").exists());
    }

    #[test]
    fn test_init_project_invalid_format() {
        let temp_dir = TempDir::new().expect("应能创建临时目录");
        let base_dir = temp_dir.path().to_str().expect("应能转换为字符串");

        let result = init_project("test_project", "invalid_format", Some(base_dir));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_project() {
        let temp_dir = TempDir::new().expect("应能创建临时目录");
        let base_dir = temp_dir.path().to_str().expect("应能转换为字符串");

        let project_path =
            init_project("test_project", "ppt169", Some(base_dir)).expect("应能创建项目");

        let result = validate_project(&project_path);
        assert!(result.is_ok());

        let validation = result.expect("应返回校验结果");
        assert!(validation.is_valid);
    }

    #[test]
    fn test_list_projects() {
        let temp_dir = TempDir::new().expect("应能创建临时目录");
        let base_dir = temp_dir.path().to_str().expect("应能转换为字符串");

        init_project("project1", "ppt169", Some(base_dir)).expect("应能创建项目1");
        init_project("project2", "ppt43", Some(base_dir)).expect("应能创建项目2");

        let result = list_projects(Some(base_dir));
        assert!(result.is_ok());

        let projects = result.expect("应返回项目列表");
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_delete_project() {
        let temp_dir = TempDir::new().expect("应能创建临时目录");
        let base_dir = temp_dir.path().to_str().expect("应能转换为字符串");

        let project_path =
            init_project("project_del", "ppt169", Some(base_dir)).expect("应能创建项目");
        let project_path_buf = PathBuf::from(&project_path);
        assert!(project_path_buf.exists());

        let result = delete_project(&project_path);
        assert!(result.is_ok());
        assert!(!project_path_buf.exists());
    }

    #[test]
    fn test_delete_project_not_found() {
        let temp_dir = TempDir::new().expect("应能创建临时目录");
        let missing = temp_dir.path().join("not_exists_project");

        let result = delete_project(&missing);
        assert!(result.is_err());
    }
}
