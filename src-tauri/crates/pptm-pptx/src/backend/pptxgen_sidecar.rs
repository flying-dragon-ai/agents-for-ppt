// PptxGenJS Sidecar 后端
// 使用 Node.js + PptxGenJS 库生成 PPTX

use crate::{PptxBackend, PptxConfig, PptxError, Result, Slide, SlideContent};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

/// PptxGenJS Sidecar 后端
///
/// 通过 Node.js sidecar 进程调用 PptxGenJS 库生成 PPTX
pub struct PptxGenSidecar {
    sidecar_path: String,
}

impl PptxGenSidecar {
    /// 创建新的 PptxGenSidecar 后端
    pub fn new() -> Self {
        Self {
            sidecar_path: "sidecars/pptxgen/index.js".to_string(),
        }
    }

    /// 设置 sidecar 路径
    pub fn with_sidecar_path(mut self, path: String) -> Self {
        self.sidecar_path = path;
        self
    }
}

impl Default for PptxGenSidecar {
    fn default() -> Self {
        Self::new()
    }
}

impl PptxBackend for PptxGenSidecar {
    fn export(&self, slides: &[Slide], output_path: &Path, config: &PptxConfig) -> Result<()> {
        // 使用 tokio runtime 执行异步操作
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PptxError::Backend(format!("无法创建 tokio runtime: {}", e)))?;

        runtime.block_on(async { self.export_async(slides, output_path, config).await })
    }

    fn name(&self) -> &str {
        "pptxgen_sidecar"
    }

    fn is_available(&self) -> bool {
        // 检查 Node.js 是否可用
        std::process::Command::new("node")
            .arg("--version")
            .output()
            .is_ok()
    }
}

impl PptxGenSidecar {
    async fn export_async(
        &self,
        slides: &[Slide],
        output_path: &Path,
        config: &PptxConfig,
    ) -> Result<()> {
        // 准备请求数据
        let request = serde_json::json!({
            "slides": slides.iter().map(|slide| {
                let content = match &slide.content {
                    SlideContent::Svg(svg) => serde_json::json!({
                        "type": "svg",
                        "data": svg,
                    }),
                    SlideContent::Png(png) => {
                        use base64::Engine;
                        serde_json::json!({
                            "type": "png",
                            "data": base64::engine::general_purpose::STANDARD.encode(png),
                        })
                    }
                };

                serde_json::json!({
                    "number": slide.number,
                    "title": slide.title,
                    "content": content,
                    "notes": slide.notes,
                })
            }).collect::<Vec<_>>(),
            "output": output_path.to_string_lossy(),
            "config": {
                "width": config.width,
                "height": config.height,
                "enableTransitions": config.enable_transitions,
                "transitionType": config.transition_type,
            },
        });

        let request_json = serde_json::to_string(&request)?;

        // 启动 Node.js sidecar 进程
        let mut child = Command::new("node")
            .arg(&self.sidecar_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| PptxError::Backend(format!("无法启动 Node.js sidecar: {}", e)))?;

        // 发送请求数据
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(request_json.as_bytes())
                .await
                .map_err(|e| PptxError::Backend(format!("无法写入 stdin: {}", e)))?;
            stdin
                .flush()
                .await
                .map_err(|e| PptxError::Backend(format!("无法 flush stdin: {}", e)))?;
        }

        // 读取响应
        let mut stdout = String::new();
        let mut stderr = String::new();

        if let Some(mut out) = child.stdout.take() {
            out.read_to_string(&mut stdout)
                .await
                .map_err(|e| PptxError::Backend(format!("无法读取 stdout: {}", e)))?;
        }

        if let Some(mut err) = child.stderr.take() {
            err.read_to_string(&mut stderr)
                .await
                .map_err(|e| PptxError::Backend(format!("无法读取 stderr: {}", e)))?;
        }

        // 等待进程结束
        let status = child
            .wait()
            .await
            .map_err(|e| PptxError::Backend(format!("等待进程失败: {}", e)))?;

        if !status.success() {
            return Err(PptxError::Backend(format!(
                "Node.js sidecar 执行失败: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        // 解析响应
        let response: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| PptxError::Backend(format!("无法解析响应: {}", e)))?;

        if let Some(error) = response.get("error") {
            return Err(PptxError::Backend(format!(
                "Sidecar 返回错误: {}",
                error.as_str().unwrap_or("未知错误")
            )));
        }

        Ok(())
    }
}
