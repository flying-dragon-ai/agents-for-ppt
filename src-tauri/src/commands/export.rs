// PPTX 导出命令

use pptm_pptx::{
    backend::{NativeOoxml, PptxGenSidecar},
    load_slides, PptxBackend, PptxConfig,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// PPTX 导出请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPptxRequest {
    /// 项目路径
    pub project_path: String,
    /// 输出文件路径（可选，默认为项目目录下的 output.pptx）
    pub output_path: Option<String>,
    /// 后端类型（"pptxgen" 或 "native"）
    pub backend: Option<String>,
    /// 画布宽度
    pub width: Option<u32>,
    /// 画布高度
    pub height: Option<u32>,
    /// 是否启用切换效果
    pub enable_transitions: Option<bool>,
    /// 切换效果类型
    pub transition_type: Option<String>,
}

/// PPTX 导出响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPptxResponse {
    /// 是否成功
    pub success: bool,
    /// 输出文件路径
    pub output_path: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 幻灯片数量
    pub slide_count: Option<usize>,
}

/// 导出 PPTX 命令
#[tauri::command]
pub async fn cmd_export_pptx(request: ExportPptxRequest) -> Result<ExportPptxResponse, String> {
    // 解析项目路径
    let project_path = PathBuf::from(&request.project_path);
    if !project_path.exists() {
        return Ok(ExportPptxResponse {
            success: false,
            output_path: None,
            error: Some(format!("项目路径不存在: {:?}", project_path)),
            slide_count: None,
        });
    }

    // 确定输出路径
    let output_path = if let Some(path) = request.output_path {
        PathBuf::from(path)
    } else {
        project_path.join("output.pptx")
    };

    // 创建配置
    let config = PptxConfig {
        width: request.width.unwrap_or(1280),
        height: request.height.unwrap_or(720),
        enable_transitions: request.enable_transitions.unwrap_or(true),
        transition_type: request.transition_type.or_else(|| Some("fade".to_string())),
    };

    // 加载幻灯片
    let slides = match load_slides(&project_path, &config) {
        Ok(slides) => slides,
        Err(e) => {
            return Ok(ExportPptxResponse {
                success: false,
                output_path: None,
                error: Some(format!("加载幻灯片失败: {}", e)),
                slide_count: None,
            });
        }
    };

    if slides.is_empty() {
        return Ok(ExportPptxResponse {
            success: false,
            output_path: None,
            error: Some("没有找到幻灯片".to_string()),
            slide_count: Some(0),
        });
    }

    // 选择后端
    let backend_name = request.backend.as_deref().unwrap_or("pptxgen");
    let result = match backend_name {
        "pptxgen" => {
            let backend = PptxGenSidecar::new();
            if !backend.is_available() {
                return Ok(ExportPptxResponse {
                    success: false,
                    output_path: None,
                    error: Some("PptxGenJS 后端不可用，请确保已安装 Node.js".to_string()),
                    slide_count: Some(slides.len()),
                });
            }
            backend.export(&slides, &output_path, &config)
        }
        "native" => {
            let backend = NativeOoxml::new();
            backend.export(&slides, &output_path, &config)
        }
        _ => {
            return Ok(ExportPptxResponse {
                success: false,
                output_path: None,
                error: Some(format!("未知的后端类型: {}", backend_name)),
                slide_count: Some(slides.len()),
            });
        }
    };

    match result {
        Ok(_) => Ok(ExportPptxResponse {
            success: true,
            output_path: Some(output_path.to_string_lossy().to_string()),
            error: None,
            slide_count: Some(slides.len()),
        }),
        Err(e) => Ok(ExportPptxResponse {
            success: false,
            output_path: None,
            error: Some(format!("导出失败: {}", e)),
            slide_count: Some(slides.len()),
        }),
    }
}

/// 检查后端可用性命令
#[tauri::command]
pub fn cmd_check_pptx_backends() -> CheckBackendsResponse {
    let pptxgen = PptxGenSidecar::new();
    let native = NativeOoxml::new();

    CheckBackendsResponse {
        pptxgen: pptxgen.is_available(),
        native: native.is_available(),
    }
}

#[derive(Debug, Serialize)]
pub struct CheckBackendsResponse {
    pub pptxgen: bool,
    pub native: bool,
}
