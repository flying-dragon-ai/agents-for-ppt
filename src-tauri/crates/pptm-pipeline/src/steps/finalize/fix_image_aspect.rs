// 图片宽高比修复模块
//
// 修复 SVG 中 <image> 元素的尺寸，使其与图片原始宽高比一致
// 这样在 PowerPoint 将 SVG 转换为形状时，图片不会被拉伸变形

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

/// 修复图片宽高比
///
/// # Arguments
///
/// * `svg_content` - SVG 文件内容
///
/// # Returns
///
/// 处理后的 SVG 内容
pub fn fix_image_aspect(svg_content: &str) -> Result<String> {
    let mut reader = Reader::from_str(svg_content);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"image" => {
                if let Ok(new_elem) = fix_image_element(&e) {
                    writer.write_event(Event::Empty(new_elem))?;
                } else {
                    writer.write_event(Event::Empty(e))?;
                }
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"image" => {
                if let Ok(new_elem) = fix_image_element(&e) {
                    writer.write_event(Event::Start(new_elem))?;
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

/// 修复单个 image 元素
fn fix_image_element(elem: &BytesStart) -> Result<BytesStart> {
    // 提取属性
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    let mut href = String::new();

    for attr in elem.attributes() {
        let attr = attr?;
        let key = std::str::from_utf8(attr.key.as_ref())?;
        let value = attr.unescape_value()?;

        match key {
            "x" => x = value.parse().unwrap_or(0.0),
            "y" => y = value.parse().unwrap_or(0.0),
            "width" => width = value.parse().unwrap_or(0.0),
            "height" => height = value.parse().unwrap_or(0.0),
            "href" | "xlink:href" => href = value.to_string(),
            _ => {}
        }
    }

    // 如果没有 href 或尺寸无效，返回原元素
    if href.is_empty() || width <= 0.0 || height <= 0.0 {
        return Ok(elem.clone());
    }

    // 获取图片尺寸
    let (img_width, img_height) = if let Some(dimensions) = get_image_dimensions(&href) {
        dimensions
    } else {
        return Ok(elem.clone());
    };

    // 计算宽高比
    let img_ratio = img_width as f32 / img_height as f32;
    let box_ratio = width / height;

    // 如果宽高比已经匹配，不需要修复
    if (img_ratio - box_ratio).abs() < 0.01 {
        return Ok(elem.clone());
    }

    // 计算新的尺寸（保持图片宽高比，居中显示）
    let (new_width, new_height, new_x, new_y) = if img_ratio > box_ratio {
        // 图片更宽，以宽度为准
        let new_height = width / img_ratio;
        let new_y = y + (height - new_height) / 2.0;
        (width, new_height, x, new_y)
    } else {
        // 图片更高，以高度为准
        let new_width = height * img_ratio;
        let new_x = x + (width - new_width) / 2.0;
        (new_width, height, new_x, y)
    };

    // 创建新元素
    let mut new_elem = BytesStart::new(std::str::from_utf8(elem.name().as_ref())?);

    for attr in elem.attributes() {
        let attr = attr?;
        let key = attr.key.as_ref();
        let key_str = std::str::from_utf8(key)?;

        match key_str {
            "x" => new_elem.push_attribute((key, format!("{:.2}", new_x).as_str())),
            "y" => new_elem.push_attribute((key, format!("{:.2}", new_y).as_str())),
            "width" => new_elem.push_attribute((key, format!("{:.2}", new_width).as_str())),
            "height" => new_elem.push_attribute((key, format!("{:.2}", new_height).as_str())),
            _ => new_elem.push_attribute(attr),
        }
    }

    Ok(new_elem)
}

/// 获取图片尺寸
fn get_image_dimensions(href: &str) -> Option<(u32, u32)> {
    if href.starts_with("data:") {
        get_image_dimensions_from_data_uri(href)
    } else {
        None
    }
}

/// 从 data URI 获取图片尺寸
fn get_image_dimensions_from_data_uri(data_uri: &str) -> Option<(u32, u32)> {
    // 解析 data URI
    let parts: Vec<&str> = data_uri.split(',').collect();
    if parts.len() != 2 {
        return None;
    }

    // 检查是否是 base64
    if !parts[0].contains("base64") {
        return None;
    }

    // 解码 base64
    let img_data = general_purpose::STANDARD.decode(parts[1]).ok()?;

    // 使用 image crate 获取尺寸
    let img = image::load_from_memory(&img_data).ok()?;
    Some((img.width(), img.height()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_image_aspect_no_images() {
        let input = r#"<svg><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let output = fix_image_aspect(input).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_get_image_dimensions_from_data_uri() {
        // 1x1 PNG
        let data_uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let dimensions = get_image_dimensions_from_data_uri(data_uri);
        assert_eq!(dimensions, Some((1, 1)));
    }

    #[test]
    fn test_fix_image_element_no_href() {
        let xml = r#"<image x="0" y="0" width="100" height="100"/>"#;
        let elem = BytesStart::from_content(xml, 5);
        let result = fix_image_element(&elem).unwrap();
        // 应该返回原元素
        assert_eq!(
            std::str::from_utf8(result.name().as_ref()).unwrap(),
            "image"
        );
    }
}
