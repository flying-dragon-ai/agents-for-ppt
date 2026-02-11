// 圆角矩形转 Path 模块
//
// 将带 rx/ry 的 <rect> 转换为等效的 <path>
// 解决 SVG 在 PowerPoint 中「转换为形状」时圆角丢失的问题

use anyhow::Result;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

/// 圆角矩形转 Path
///
/// # Arguments
///
/// * `svg_content` - SVG 文件内容
///
/// # Returns
///
/// 处理后的 SVG 内容
pub fn rect_to_path(svg_content: &str) -> Result<String> {
    let mut reader = Reader::from_str(svg_content);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"rect" => {
                if let Ok(new_elem) = convert_rect_to_path(&e) {
                    writer.write_event(Event::Empty(new_elem))?;
                } else {
                    writer.write_event(Event::Empty(e))?;
                }
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"rect" => {
                if let Ok(new_elem) = convert_rect_to_path(&e) {
                    writer.write_event(Event::Empty(new_elem))?;
                    // 跳过 rect 的结束标签
                    loop {
                        match reader.read_event_into(&mut buf) {
                            Ok(Event::End(end)) if end.name().as_ref() == b"rect" => break,
                            Ok(Event::Eof) => break,
                            _ => {}
                        }
                        buf.clear();
                    }
                } else {
                    writer.write_event(Event::Start(e))?;
                }
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

/// 将圆角矩形转换为 path
fn convert_rect_to_path(elem: &BytesStart) -> Result<BytesStart> {
    // 提取属性
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    let mut rx = 0.0f32;
    let mut ry = 0.0f32;

    for attr in elem.attributes() {
        let attr = attr?;
        let key = std::str::from_utf8(attr.key.as_ref())?;
        let value = attr.unescape_value()?;

        match key {
            "x" => x = parse_float(&value),
            "y" => y = parse_float(&value),
            "width" => width = parse_float(&value),
            "height" => height = parse_float(&value),
            "rx" => rx = parse_float(&value),
            "ry" => ry = parse_float(&value),
            _ => {}
        }
    }

    // 如果只指定了一个，另一个取相同值
    if rx == 0.0 && ry > 0.0 {
        rx = ry;
    } else if ry == 0.0 && rx > 0.0 {
        ry = rx;
    }

    // 如果没有圆角，返回原元素
    if rx <= 0.0 && ry <= 0.0 {
        return Ok(elem.clone());
    }

    // 如果尺寸无效，返回原元素
    if width <= 0.0 || height <= 0.0 {
        return Ok(elem.clone());
    }

    // 生成 path
    let path_d = rect_to_rounded_path(x, y, width, height, rx, ry);

    // 创建新的 path 元素
    let mut new_elem = BytesStart::new("path");

    // 添加 d 属性
    new_elem.push_attribute(("d", path_d.as_str()));

    // 复制其他属性（排除 rect 特有的属性）
    for attr in elem.attributes() {
        let attr = attr?;
        let key = std::str::from_utf8(attr.key.as_ref())?;

        if !matches!(key, "x" | "y" | "width" | "height" | "rx" | "ry") {
            new_elem.push_attribute(attr);
        }
    }

    Ok(new_elem)
}

/// 将圆角矩形转换为 SVG path 字符串
fn rect_to_rounded_path(x: f32, y: f32, width: f32, height: f32, rx: f32, ry: f32) -> String {
    // 限制圆角半径不超过宽高的一半
    let rx = rx.min(width / 2.0);
    let ry = ry.min(height / 2.0);

    // 计算关键点
    let x1 = x + rx;
    let x2 = x + width - rx;
    let y1 = y + ry;
    let y2 = y + height - ry;

    // 构建 path
    format!(
        "M{:.2},{:.2} H{:.2} A{:.2},{:.2} 0 0 1 {:.2},{:.2} V{:.2} A{:.2},{:.2} 0 0 1 {:.2},{:.2} H{:.2} A{:.2},{:.2} 0 0 1 {:.2},{:.2} V{:.2} A{:.2},{:.2} 0 0 1 {:.2},{:.2} Z",
        x1, y,
        x2,
        rx, ry, x + width, y1,
        y2,
        rx, ry, x2, y + height,
        x1,
        rx, ry, x, y2,
        y1,
        rx, ry, x1, y
    )
    .replace(".00", "")
}

/// 安全解析浮点数
fn parse_float(val: &str) -> f32 {
    // 移除单位
    let val = val.trim();
    let val = val
        .trim_end_matches("px")
        .trim_end_matches("pt")
        .trim_end_matches("em")
        .trim_end_matches("%")
        .trim_end_matches("rem");

    val.parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_to_path_no_rounded() {
        let input = r#"<svg><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let output = rect_to_path(input).unwrap();
        // 没有圆角，应该保持为 rect
        assert!(output.contains("<rect"));
    }

    #[test]
    fn test_rect_to_path_with_rounded() {
        let input = r#"<svg><rect x="0" y="0" width="100" height="100" rx="10"/></svg>"#;
        let output = rect_to_path(input).unwrap();
        // 应该转换为 path
        assert!(output.contains("<path"));
        assert!(output.contains("d="));
        // 不应该包含 rect
        assert!(!output.contains("<rect"));
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_float("100"), 100.0);
        assert_eq!(parse_float("100.5"), 100.5);
        assert_eq!(parse_float("100px"), 100.0);
        assert_eq!(parse_float("100pt"), 100.0);
        assert_eq!(parse_float("invalid"), 0.0);
    }

    #[test]
    fn test_rect_to_rounded_path() {
        let path = rect_to_rounded_path(0.0, 0.0, 100.0, 100.0, 10.0, 10.0);
        assert!(path.starts_with("M"));
        assert!(path.contains("A"));
        assert!(path.ends_with("Z"));
    }
}
