use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 画布格式定义（与 Python `tools/project_utils.py` 保持一致）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanvasFormat {
    /// 格式键，例如 `ppt169`。
    pub key: String,
    /// 画布名称。
    pub name: String,
    /// 尺寸描述，例如 `1280x720`。
    pub dimensions: String,
    /// SVG viewBox。
    pub viewbox: String,
    /// 宽度（像素）。
    pub width: u32,
    /// 高度（像素）。
    pub height: u32,
    /// 长宽比。
    pub aspect_ratio: String,
    /// 使用场景分类。
    pub category: String,
}

lazy_static! {
    /// 支持的画布格式（统一数据源）。
    pub static ref CANVAS_FORMATS: HashMap<String, CanvasFormat> = {
        let mut formats = HashMap::new();
        formats.insert(
            "ppt169".to_string(),
            CanvasFormat {
                key: "ppt169".to_string(),
                name: "PPT 16:9".to_string(),
                dimensions: "1280x720".to_string(),
                viewbox: "0 0 1280 720".to_string(),
                width: 1280,
                height: 720,
                aspect_ratio: "16:9".to_string(),
                category: "presentation".to_string(),
            },
        );
        formats.insert(
            "ppt43".to_string(),
            CanvasFormat {
                key: "ppt43".to_string(),
                name: "PPT 4:3".to_string(),
                dimensions: "1024x768".to_string(),
                viewbox: "0 0 1024 768".to_string(),
                width: 1024,
                height: 768,
                aspect_ratio: "4:3".to_string(),
                category: "presentation".to_string(),
            },
        );
        formats.insert(
            "wechat".to_string(),
            CanvasFormat {
                key: "wechat".to_string(),
                name: "微信公众号头图".to_string(),
                dimensions: "900x383".to_string(),
                viewbox: "0 0 900 383".to_string(),
                width: 900,
                height: 383,
                aspect_ratio: "2.35:1".to_string(),
                category: "marketing".to_string(),
            },
        );
        formats.insert(
            "xiaohongshu".to_string(),
            CanvasFormat {
                key: "xiaohongshu".to_string(),
                name: "小红书".to_string(),
                dimensions: "1242x1660".to_string(),
                viewbox: "0 0 1242 1660".to_string(),
                width: 1242,
                height: 1660,
                aspect_ratio: "3:4".to_string(),
                category: "social".to_string(),
            },
        );
        formats.insert(
            "moments".to_string(),
            CanvasFormat {
                key: "moments".to_string(),
                name: "朋友圈/Instagram".to_string(),
                dimensions: "1080x1080".to_string(),
                viewbox: "0 0 1080 1080".to_string(),
                width: 1080,
                height: 1080,
                aspect_ratio: "1:1".to_string(),
                category: "social".to_string(),
            },
        );
        formats.insert(
            "story".to_string(),
            CanvasFormat {
                key: "story".to_string(),
                name: "Story/竖版".to_string(),
                dimensions: "1080x1920".to_string(),
                viewbox: "0 0 1080 1920".to_string(),
                width: 1080,
                height: 1920,
                aspect_ratio: "9:16".to_string(),
                category: "social".to_string(),
            },
        );
        formats.insert(
            "banner".to_string(),
            CanvasFormat {
                key: "banner".to_string(),
                name: "横版 Banner".to_string(),
                dimensions: "1920x1080".to_string(),
                viewbox: "0 0 1920 1080".to_string(),
                width: 1920,
                height: 1080,
                aspect_ratio: "16:9".to_string(),
                category: "marketing".to_string(),
            },
        );
        formats.insert(
            "a4".to_string(),
            CanvasFormat {
                key: "a4".to_string(),
                name: "A4 打印".to_string(),
                dimensions: "1240x1754".to_string(),
                viewbox: "0 0 1240 1754".to_string(),
                width: 1240,
                height: 1754,
                aspect_ratio: "1:1.414".to_string(),
                category: "document".to_string(),
            },
        );
        formats
    };

    /// 画布格式别名映射（与 Python 逻辑一致）。
    static ref CANVAS_FORMAT_ALIASES: HashMap<&'static str, &'static str> = {
        let mut aliases = HashMap::new();
        aliases.insert("xhs", "xiaohongshu");
        aliases.insert("wechat_moment", "moments");
        aliases.insert("wechat-moment", "moments");
        aliases.insert("朋友圈", "moments");
        aliases.insert("小红书", "xiaohongshu");
        aliases
    };
}

/// 标准化画布格式键名（支持常见别名）。
pub fn normalize_canvas_format(format_key: &str) -> String {
    if format_key.trim().is_empty() {
        return String::new();
    }

    let key = format_key.trim().to_lowercase();
    CANVAS_FORMAT_ALIASES
        .get(key.as_str())
        .copied()
        .unwrap_or(key.as_str())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_formats_count() {
        assert_eq!(CANVAS_FORMATS.len(), 8);
    }

    #[test]
    fn test_canvas_format_content() {
        let ppt169 = CANVAS_FORMATS
            .get("ppt169")
            .expect("应存在 ppt169 画布格式");
        assert_eq!(ppt169.width, 1280);
        assert_eq!(ppt169.height, 720);
        assert_eq!(ppt169.viewbox, "0 0 1280 720");
        assert_eq!(ppt169.aspect_ratio, "16:9");
    }

    #[test]
    fn test_normalize_canvas_format_for_aliases() {
        assert_eq!(normalize_canvas_format("xhs"), "xiaohongshu");
        assert_eq!(normalize_canvas_format("wechat_moment"), "moments");
        assert_eq!(normalize_canvas_format("wechat-moment"), "moments");
        assert_eq!(normalize_canvas_format("朋友圈"), "moments");
        assert_eq!(normalize_canvas_format("小红书"), "xiaohongshu");
    }

    #[test]
    fn test_normalize_canvas_format_trim_and_case() {
        assert_eq!(normalize_canvas_format("  PPT169  "), "ppt169");
        assert_eq!(normalize_canvas_format("  WeChat_Moment "), "moments");
    }

    #[test]
    fn test_normalize_canvas_format_for_empty_or_unknown() {
        assert_eq!(normalize_canvas_format(""), "");
        assert_eq!(normalize_canvas_format("   "), "");
        assert_eq!(normalize_canvas_format("custom-format"), "custom-format");
    }
}
