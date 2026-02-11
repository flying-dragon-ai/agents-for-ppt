// PPTX 导出模块
// 支持双后端：pptxgen_sidecar (Node.js) 和 native_ooxml (Rust)

pub mod backend;

use std::path::Path;
use thiserror::Error;

/// PPTX 导出错误
#[derive(Debug, Error)]
pub enum PptxError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("SVG 解析错误: {0}")]
    SvgParse(String),

    #[error("PNG 转换错误: {0}")]
    PngConversion(String),

    #[error("后端错误: {0}")]
    Backend(String),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Zip 错误: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub type Result<T> = std::result::Result<T, PptxError>;

/// 幻灯片内容类型
#[derive(Debug, Clone)]
pub enum SlideContent {
    /// SVG 内容（优先使用）
    Svg(String),
    /// PNG 内容（fallback）
    Png(Vec<u8>),
}

/// 幻灯片数据
#[derive(Debug, Clone)]
pub struct Slide {
    /// 幻灯片编号（从 1 开始）
    pub number: usize,
    /// 幻灯片标题
    pub title: String,
    /// 幻灯片内容
    pub content: SlideContent,
    /// 演讲备注（Markdown 格式）
    pub notes: Option<String>,
}

/// PPTX 导出配置
#[derive(Debug, Clone)]
pub struct PptxConfig {
    /// 画布宽度（像素）
    pub width: u32,
    /// 画布高度（像素）
    pub height: u32,
    /// 是否启用切换效果
    pub enable_transitions: bool,
    /// 切换效果类型（如 "fade", "push" 等）
    pub transition_type: Option<String>,
}

impl Default for PptxConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            enable_transitions: true,
            transition_type: Some("fade".to_string()),
        }
    }
}

/// PPTX 后端 trait
///
/// 定义统一的 PPTX 导出接口，支持不同的实现后端
pub trait PptxBackend: Send + Sync {
    /// 导出 PPTX 文件
    ///
    /// # 参数
    /// - `slides`: 幻灯片列表
    /// - `output_path`: 输出文件路径
    /// - `config`: 导出配置
    fn export(&self, slides: &[Slide], output_path: &Path, config: &PptxConfig) -> Result<()>;

    /// 获取后端名称
    fn name(&self) -> &str;

    /// 检查后端是否可用
    fn is_available(&self) -> bool;
}

/// SVG 转 PNG 工具
///
/// 使用 resvg 将 SVG 转换为 PNG，用于不兼容的 SVG 内容
pub fn svg_to_png(svg_content: &str, width: u32, height: u32) -> Result<Vec<u8>> {
    use usvg::TreeParsing;

    // 解析 SVG
    let opt = usvg::Options::default();
    let tree =
        usvg::Tree::from_str(svg_content, &opt).map_err(|e| PptxError::SvgParse(e.to_string()))?;

    // 创建 pixmap
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| PptxError::PngConversion("无法创建 pixmap".to_string()))?;

    // 渲染 SVG
    resvg::render(&tree, usvg::Transform::default(), &mut pixmap.as_mut());

    // 编码为 PNG
    pixmap
        .encode_png()
        .map_err(|e| PptxError::PngConversion(e.to_string()))
}

/// 读取演讲备注
///
/// 从 notes 目录读取 Markdown 格式的演讲备注
pub fn read_notes(
    notes_dir: &Path,
    slide_number: usize,
    slide_title: &str,
) -> Result<Option<String>> {
    // 尝试多种文件名格式
    let patterns = vec![
        format!("{:02}_{}.md", slide_number, slide_title),
        format!("{}_{}.md", slide_number, slide_title),
        format!("{:02}.md", slide_number),
        format!("{}.md", slide_number),
    ];

    for pattern in patterns {
        let path = notes_dir.join(&pattern);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            return Ok(Some(content));
        }
    }

    Ok(None)
}

/// 加载项目的所有幻灯片
///
/// 从 svg_final 目录加载 SVG 文件，从 notes 目录加载演讲备注
pub fn load_slides(project_path: &Path, config: &PptxConfig) -> Result<Vec<Slide>> {
    let svg_dir = project_path.join("svg_final");
    let notes_dir = project_path.join("notes");

    if !svg_dir.exists() {
        return Err(PptxError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("SVG 目录不存在: {:?}", svg_dir),
        )));
    }

    let mut slides = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&svg_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("svg"))
                .unwrap_or(false)
        })
        .collect();

    // 按文件名排序
    entries.sort_by_key(|e| e.file_name());

    for (index, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let svg_content = std::fs::read_to_string(&path)?;

        // 提取标题（从文件名）
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        let slide_number = index + 1;

        // 读取演讲备注
        let notes = if notes_dir.exists() {
            read_notes(&notes_dir, slide_number, &title)?
        } else {
            None
        };

        // 尝试使用 SVG，如果失败则转换为 PNG
        let content = match validate_svg(&svg_content) {
            Ok(_) => SlideContent::Svg(svg_content),
            Err(_) => {
                // SVG 不兼容，转换为 PNG
                let png_data = svg_to_png(&svg_content, config.width, config.height)?;
                SlideContent::Png(png_data)
            }
        };

        slides.push(Slide {
            number: slide_number,
            title,
            content,
            notes,
        });
    }

    Ok(slides)
}

/// 验证 SVG 是否兼容 PPTX
///
/// 检查 SVG 是否包含 PPTX 不支持的特性
fn validate_svg(svg_content: &str) -> Result<()> {
    // 检查黑名单特性
    let blacklist = vec![
        "clipPath",
        "mask",
        "<style",
        "class=",
        "foreignObject",
        "textPath",
        "@font-face",
        "animate",
        "marker-end",
    ];

    for feature in blacklist {
        if svg_content.contains(feature) {
            return Err(PptxError::SvgParse(format!(
                "SVG 包含不兼容的特性: {}",
                feature
            )));
        }
    }

    Ok(())
}
