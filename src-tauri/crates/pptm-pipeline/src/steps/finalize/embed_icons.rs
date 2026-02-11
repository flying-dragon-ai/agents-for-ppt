// 图标嵌入模块
//
// 将 SVG 文件中的图标占位符替换为实际的图标代码
// 占位符语法：<use data-icon="rocket" x="100" y="200" width="48" height="48" fill="#0076A8"/>

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use regex::Regex;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

/// 图标基础尺寸
const ICON_BASE_SIZE: f32 = 16.0;

/// 图标库（从 templates/icons/ 加载）
lazy_static! {
    static ref ICON_LIBRARY: HashMap<String, Vec<String>> = load_icon_library();
}

/// 从 templates/icons/ 目录加载所有图标
fn load_icon_library() -> HashMap<String, Vec<String>> {
    let mut icons = HashMap::new();

    // 获取图标目录路径
    let icons_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("templates").join("icons"));

    if let Some(icons_dir) = icons_dir {
        if icons_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&icons_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "svg") {
                        if let Some(icon_name) = path.file_stem() {
                            if let Ok(paths) = extract_paths_from_icon(&path) {
                                icons.insert(icon_name.to_string_lossy().to_string(), paths);
                            }
                        }
                    }
                }
            }
        }
    }

    icons
}

/// 从图标 SVG 文件中提取所有 path 元素
fn extract_paths_from_icon(icon_path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(icon_path)
        .with_context(|| format!("Failed to read icon file: {}", icon_path.display()))?;

    let re = Regex::new(r#"<path\s+([^>]*)/>"#).unwrap();
    let fill_re = Regex::new(r#"\s*fill="[^"]*""#).unwrap();

    let mut paths = Vec::new();
    for cap in re.captures_iter(&content) {
        if let Some(attrs) = cap.get(1) {
            // 移除 fill 属性（将在外层 <g> 上统一设置）
            let attrs_clean = fill_re.replace_all(attrs.as_str(), "");
            paths.push(format!("<path {}/>", attrs_clean.trim()));
        }
    }

    Ok(paths)
}

/// 解析 use 元素的属性
#[derive(Debug, Default)]
struct UseAttrs {
    icon: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill: Option<String>,
}

impl UseAttrs {
    fn from_element(elem: &BytesStart) -> Result<Self> {
        let mut attrs = UseAttrs::default();

        for attr in elem.attributes() {
            let attr = attr?;
            let key = std::str::from_utf8(attr.key.as_ref())?;
            let value = attr.unescape_value()?;

            match key {
                "data-icon" => attrs.icon = value.to_string(),
                "x" => attrs.x = value.parse().unwrap_or(0.0),
                "y" => attrs.y = value.parse().unwrap_or(0.0),
                "width" => attrs.width = value.parse().unwrap_or(0.0),
                "height" => attrs.height = value.parse().unwrap_or(0.0),
                "fill" => attrs.fill = Some(value.to_string()),
                _ => {}
            }
        }

        Ok(attrs)
    }

    /// 生成图标的 <g> 元素
    fn generate_icon_group(&self, paths: &[String]) -> String {
        let scale = self.width / ICON_BASE_SIZE;
        let mut group = format!(
            r#"<g transform="translate({}, {}) scale({})"#,
            self.x, self.y, scale
        );

        if let Some(fill) = &self.fill {
            group.push_str(&format!(r#" fill="{}""#, fill));
        }

        group.push('>');

        for path in paths {
            group.push_str(path);
        }

        group.push_str("</g>");
        group
    }
}

/// 嵌入图标到 SVG 内容中
///
/// # Arguments
///
/// * `svg_content` - SVG 文件内容
///
/// # Returns
///
/// 处理后的 SVG 内容
pub fn embed_icons(svg_content: &str) -> Result<String> {
    let mut reader = Reader::from_str(svg_content);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"use" => {
                // 检查是否有 data-icon 属性
                let has_data_icon = e
                    .attributes()
                    .any(|attr| attr.map_or(false, |a| a.key.as_ref() == b"data-icon"));

                if has_data_icon {
                    // 解析属性
                    if let Ok(attrs) = UseAttrs::from_element(&e) {
                        // 查找图标定义
                        if let Some(paths) = ICON_LIBRARY.get(&attrs.icon) {
                            // 替换为内联 SVG
                            let icon_group = attrs.generate_icon_group(paths);
                            writer.write_event(Event::Text(BytesText::new(&icon_group)))?;
                            buf.clear();
                            continue;
                        }
                    }
                }

                // 如果没有找到图标或解析失败，保持原样
                writer.write_event(Event::Empty(e))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => writer.write_event(e)?,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to parse SVG: {}", e));
            }
        }

        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_icons_no_icons() {
        let input = r#"<svg><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let output = embed_icons(input).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_embed_icons_with_placeholder() {
        let input = r##"<svg><use data-icon="arrow" x="100" y="200" width="48" height="48" fill="#0076A8"/></svg>"##;
        let output = embed_icons(input).unwrap();

        // 应该包含 <g> 元素
        assert!(output.contains("<g transform="));
        // 不应该包含 data-icon
        assert!(!output.contains("data-icon"));
    }

    #[test]
    fn test_use_attrs_from_element() {
        let xml = r##"<use data-icon="rocket" x="100" y="200" width="48" height="48" fill="#0076A8"/>"##;
        let elem = BytesStart::from_content(xml, 3);
        let attrs = UseAttrs::from_element(&elem).unwrap();

        assert_eq!(attrs.icon, "rocket");
        assert_eq!(attrs.x, 100.0);
        assert_eq!(attrs.y, 200.0);
        assert_eq!(attrs.width, 48.0);
        assert_eq!(attrs.height, 48.0);
        assert_eq!(attrs.fill, Some("#0076A8".to_string()));
    }
}
