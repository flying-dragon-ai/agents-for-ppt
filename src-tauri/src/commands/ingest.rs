use crate::state::AppState;
use std::path::PathBuf;
use tauri::State;

/// PDF 转 Markdown 命令
///
/// # 参数
/// - `pdf_path`: PDF 文件路径
/// - `output_path`: 输出 Markdown 文件路径
///
/// # 返回
/// 成功返回输出文件路径，失败返回错误信息
///
/// # 错误
/// - PDF 文件不存在
/// - PDF 解析失败
/// - 文件写入失败
#[tauri::command]
pub async fn cmd_pdf_to_md(
    _state: State<'_, AppState>,
    pdf_path: String,
    output_path: String,
) -> Result<String, String> {
    let pdf_path = PathBuf::from(pdf_path);
    let output_path = PathBuf::from(output_path);

    // 验证 PDF 文件存在
    if !pdf_path.exists() {
        return Err(format!("PDF 文件不存在: {}", pdf_path.display()));
    }

    // 验证 PDF 文件扩展名
    if pdf_path.extension().and_then(|s| s.to_str()) != Some("pdf") {
        return Err(format!("不是有效的 PDF 文件: {}", pdf_path.display()));
    }

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建输出目录失败: {}", e))?;
    }

    // 调用 PDF 转换函数
    pptm_pipeline::steps::pdf_to_md::pdf_to_md(&pdf_path, &output_path)
        .map_err(|e| format!("PDF 转换失败: {}", e))?;

    Ok(output_path.display().to_string())
}

/// 网页转 Markdown 命令
///
/// # 参数
/// - `url`: 网页 URL
/// - `output_path`: 输出 Markdown 文件路径
/// - `use_sidecar`: 是否使用 sidecar（处理 JavaScript 渲染的页面）
///
/// # 返回
/// 成功返回输出文件路径，失败返回错误信息
///
/// # 错误
/// - URL 格式无效
/// - 网页获取失败
/// - HTML 解析失败
/// - 文件写入失败
#[tauri::command]
pub async fn cmd_web_to_md(
    _state: State<'_, AppState>,
    url: String,
    output_path: String,
    use_sidecar: Option<bool>,
) -> Result<String, String> {
    let output_path = PathBuf::from(output_path);

    // 验证 URL 格式
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("无效的 URL 格式，必须以 http:// 或 https:// 开头".to_string());
    }

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建输出目录失败: {}", e))?;
    }

    // 选择转换方法
    if use_sidecar.unwrap_or(false) {
        // 使用 sidecar（处理复杂网页）
        pptm_pipeline::steps::web_to_md::web_to_md_with_sidecar(&url, &output_path)
            .await
            .map_err(|e| format!("网页转换失败（sidecar）: {}", e))?;
    } else {
        // 使用普通方法
        pptm_pipeline::steps::web_to_md::web_to_md(&url, &output_path)
            .await
            .map_err(|e| format!("网页转换失败: {}", e))?;
    }

    Ok(output_path.display().to_string())
}

/// 批量转换 PDF
#[tauri::command]
pub async fn cmd_batch_pdf_to_md(
    _state: State<'_, AppState>,
    pdf_paths: Vec<String>,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let output_dir = PathBuf::from(output_dir);

    // 确保输出目录存在
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;

    let mut results = Vec::new();

    for pdf_path in pdf_paths {
        let pdf_path = PathBuf::from(pdf_path);
        let file_name = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let output_path = output_dir.join(format!("{}.md", file_name));

        match pptm_pipeline::steps::pdf_to_md::pdf_to_md(&pdf_path, &output_path) {
            Ok(_) => results.push(output_path.display().to_string()),
            Err(e) => {
                eprintln!("转换失败 {}: {}", pdf_path.display(), e);
                continue;
            }
        }
    }

    if results.is_empty() {
        return Err("所有 PDF 转换均失败".to_string());
    }

    Ok(results)
}
