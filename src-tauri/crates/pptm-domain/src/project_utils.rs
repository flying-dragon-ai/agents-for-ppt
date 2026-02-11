use crate::config::{normalize_canvas_format, CanvasFormat, CANVAS_FORMATS};
use chrono::NaiveDate;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 项目名称解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedProjectName {
    /// 项目名称（不含格式和日期）
    pub name: String,
    /// 画布格式键
    pub format: String,
    /// 画布格式名称
    pub format_name: String,
    /// 日期字符串（YYYYMMDD）
    pub date: String,
    /// 格式化的日期（YYYY-MM-DD）
    pub date_formatted: String,
}

/// 项目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// 项目路径
    pub path: String,
    /// 目录名
    pub dir_name: String,
    /// 项目名称
    pub name: String,
    /// 画布格式键
    pub format: String,
    /// 画布格式名称
    pub format_name: String,
    /// 日期字符串（YYYYMMDD）
    pub date: String,
    /// 格式化的日期（YYYY-MM-DD）
    pub date_formatted: String,
    /// 项目是否存在
    pub exists: bool,
    /// SVG 文件数量
    pub svg_count: usize,
    /// 是否有设计规范文件
    pub has_spec: bool,
    /// 是否有 README
    pub has_readme: bool,
    /// 是否有来源文档
    pub has_source: bool,
    /// 设计规范文件名
    pub spec_file: Option<String>,
    /// SVG 文件列表
    pub svg_files: Vec<String>,
    /// 画布信息
    pub canvas_info: Option<CanvasFormat>,
}

/// 项目验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 错误列表
    pub errors: Vec<String>,
    /// 警告列表
    pub warnings: Vec<String>,
}

/// 从项目目录名解析项目信息
pub fn parse_project_name(dir_name: &str) -> ParsedProjectName {
    let mut result = ParsedProjectName {
        name: dir_name.to_string(),
        format: "unknown".to_string(),
        format_name: "未知格式".to_string(),
        date: "unknown".to_string(),
        date_formatted: "未知日期".to_string(),
    };

    let dir_name_lower = dir_name.to_lowercase();

    // 提取日期 (格式: _YYYYMMDD)
    let date_regex = Regex::new(r"_(\d{8})$").unwrap();
    if let Some(captures) = date_regex.captures(dir_name) {
        let date_str = captures.get(1).unwrap().as_str();
        result.date = date_str.to_string();

        // 尝试格式化日期
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y%m%d") {
            result.date_formatted = date.format("%Y-%m-%d").to_string();
        }
    }

    // 优先按标准格式解析: name_format_YYYYMMDD
    let full_regex = Regex::new(r"^(?P<name>.+)_(?P<format>[a-z0-9_-]+)_(?P<date>\d{8})$").unwrap();
    if let Some(captures) = full_regex.captures(&dir_name_lower) {
        let raw_format = captures.name("format").unwrap().as_str();
        let normalized_format = normalize_canvas_format(raw_format);

        if CANVAS_FORMATS.contains_key(&normalized_format) {
            result.format = normalized_format.clone();
            result.format_name = CANVAS_FORMATS.get(&normalized_format).unwrap().name.clone();
            result.name = captures.name("name").unwrap().as_str().to_string();
            return result;
        }
    }

    // 兜底：只匹配末尾 `_format`，避免误删项目名内部片段
    let mut sorted_formats: Vec<_> = CANVAS_FORMATS.keys().collect();
    sorted_formats.sort_by_key(|b| std::cmp::Reverse(b.len())); // 按长度降序

    for fmt_key in sorted_formats {
        let pattern = format!(r"_{}(?:_\d{{8}})?$", regex::escape(fmt_key));
        let fmt_regex = Regex::new(&pattern).unwrap();
        if fmt_regex.is_match(&dir_name_lower) {
            result.format = fmt_key.clone();
            result.format_name = CANVAS_FORMATS.get(fmt_key).unwrap().name.clone();
            break;
        }
    }

    // 提取项目名称（仅移除末尾日期和格式后缀）
    let mut name = date_regex.replace(dir_name, "").to_string();
    if result.format != "unknown" {
        let format_pattern = format!(r"_{}$", regex::escape(&result.format));
        let format_regex = Regex::new(&format_pattern).unwrap();
        name = format_regex.replace(&name, "").to_string();
    }
    result.name = name;

    result
}

/// 获取项目的详细信息
pub fn get_project_info<P: AsRef<Path>>(project_path: P) -> ProjectInfo {
    let project_path = project_path.as_ref();
    let dir_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    // 解析目录名
    let parsed = parse_project_name(&dir_name);

    let mut info = ProjectInfo {
        path: project_path.to_string_lossy().to_string(),
        dir_name: dir_name.clone(),
        name: parsed.name,
        format: parsed.format.clone(),
        format_name: parsed.format_name,
        date: parsed.date,
        date_formatted: parsed.date_formatted,
        exists: project_path.exists(),
        svg_count: 0,
        has_spec: false,
        has_readme: false,
        has_source: false,
        spec_file: None,
        svg_files: Vec::new(),
        canvas_info: None,
    };

    if !project_path.exists() {
        return info;
    }

    // 检查 README.md
    info.has_readme = project_path.join("README.md").exists();

    // 检查设计规范文件（多个可能的名称）
    let spec_files = vec![
        "设计规范与内容大纲.md",
        "design_specification.md",
        "设计规范.md",
    ];
    for spec_file in spec_files {
        if project_path.join(spec_file).exists() {
            info.has_spec = true;
            info.spec_file = Some(spec_file.to_string());
            break;
        }
    }

    // 检查来源文档
    info.has_source = project_path.join("来源文档.md").exists();

    // 统计 SVG 文件
    let svg_output = project_path.join("svg_output");
    if svg_output.exists() {
        if let Ok(entries) = std::fs::read_dir(&svg_output) {
            let mut svg_files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("svg"))
                        .unwrap_or(false)
                })
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();

            svg_files.sort();
            info.svg_count = svg_files.len();
            info.svg_files = svg_files;
        }
    }

    // 获取画布格式详细信息
    if let Some(canvas_format) = CANVAS_FORMATS.get(&info.format) {
        info.canvas_info = Some(canvas_format.clone());
    }

    info
}

/// 验证项目结构的完整性
pub fn validate_project_structure<P: AsRef<Path>>(
    project_path: P,
    _verbose: bool,
) -> ValidationResult {
    let project_path = project_path.as_ref();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 检查目录是否存在
    if !project_path.exists() {
        let msg = format!("项目目录不存在: {}", project_path.display());
        errors.push(msg);
        return ValidationResult {
            is_valid: false,
            errors,
            warnings,
        };
    }

    if !project_path.is_dir() {
        errors.push(format!("不是有效的目录: {}", project_path.display()));
        return ValidationResult {
            is_valid: false,
            errors,
            warnings,
        };
    }

    // 检查必需文件
    if !project_path.join("README.md").exists() {
        let msg = "缺少必需文件: README.md".to_string();
        errors.push(msg);
    }

    // 检查设计规范文件
    let spec_files = [
        "设计规范与内容大纲.md",
        "design_specification.md",
        "设计规范.md",
    ];
    let has_spec = spec_files.iter().any(|f| project_path.join(f).exists());

    if !has_spec {
        let msg = "缺少设计规范文件（建议文件名: 设计规范与内容大纲.md）".to_string();
        warnings.push(msg);
    }

    // 检查 svg_output 目录
    let svg_output = project_path.join("svg_output");
    if !svg_output.exists() {
        errors.push("缺少 svg_output 目录".to_string());
    } else if !svg_output.is_dir() {
        errors.push("svg_output 不是目录".to_string());
    } else {
        // 检查是否有 SVG 文件
        if let Ok(entries) = std::fs::read_dir(&svg_output) {
            let svg_files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("svg"))
                        .unwrap_or(false)
                })
                .collect();

            if svg_files.is_empty() {
                warnings.push("svg_output 目录为空，没有 SVG 文件".to_string());
            } else {
                // 验证 SVG 文件命名
                let naming_regex = Regex::new(r"^(slide_\d+_\w+|P?\d+_.+)\.svg$").unwrap();
                for entry in svg_files {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if !naming_regex.is_match(&file_name) {
                        warnings.push(format!("SVG 文件命名不规范: {}", file_name));
                    }
                }
            }
        }
    }

    // 检查目录命名格式
    let dir_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let date_regex = Regex::new(r"_\d{8}$").unwrap();
    if !date_regex.is_match(dir_name) {
        warnings.push(format!("目录名缺少日期后缀 (_YYYYMMDD): {}", dir_name));
    }

    let is_valid = errors.is_empty();
    ValidationResult {
        is_valid,
        errors,
        warnings,
    }
}

/// 查找指定目录下的所有项目
pub fn find_all_projects<P: AsRef<Path>>(base_dir: P) -> Vec<PathBuf> {
    let base_path = base_dir.as_ref();
    if !base_path.exists() {
        return Vec::new();
    }

    let mut projects = Vec::new();

    if let Ok(entries) = std::fs::read_dir(base_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // 跳过隐藏目录
                if dir_name.starts_with('.') {
                    continue;
                }

                // 检查是否是有效的项目目录（包含 svg_output 或设计规范）
                let has_svg_output = path.join("svg_output").exists();
                let has_spec = [
                    "设计规范与内容大纲.md",
                    "design_specification.md",
                    "设计规范.md",
                ]
                .iter()
                .any(|f| path.join(f).exists());

                if has_svg_output || has_spec {
                    projects.push(path);
                }
            }
        }
    }

    projects.sort();
    projects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_name_standard() {
        let result = parse_project_name("myproject_ppt169_20260211");
        assert_eq!(result.name, "myproject");
        assert_eq!(result.format, "ppt169");
        assert_eq!(result.format_name, "PPT 16:9");
        assert_eq!(result.date, "20260211");
        assert_eq!(result.date_formatted, "2026-02-11");
    }

    #[test]
    fn test_parse_project_name_with_alias() {
        let result = parse_project_name("test_xhs_20260211");
        assert_eq!(result.name, "test");
        assert_eq!(result.format, "xiaohongshu");
        assert_eq!(result.format_name, "小红书");
    }

    #[test]
    fn test_parse_project_name_unknown_format() {
        let result = parse_project_name("myproject_20260211");
        assert_eq!(result.name, "myproject");
        assert_eq!(result.format, "unknown");
        assert_eq!(result.date, "20260211");
    }

    #[test]
    fn test_parse_project_name_no_date() {
        let result = parse_project_name("myproject_ppt169");
        assert_eq!(result.name, "myproject");
        assert_eq!(result.format, "ppt169");
        assert_eq!(result.date, "unknown");
    }
}
