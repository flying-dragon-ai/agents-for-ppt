// 图片嵌入模块
//
// 将 SVG 中引用的外部图片转换为 Base64 内嵌格式

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;
use std::path::Path;

/// 根据文件扩展名返回 MIME 类型
fn get_mime_type(filename: &str) -> &'static str {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

/// 嵌入图片到 SVG 内容中
///
/// # Arguments
///
/// * `svg_content` - SVG 文件内容
/// * `project_path` - 项目目录路径（用于解析相对路径）
///
/// # Returns
///
/// 处理后的 SVG 内容
pub fn embed_images(svg_content: &str, project_path: &Path) -> Result<String> {
    let mut reader = Reader::from_str(svg_content);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"image" => {
                let mut new_elem = e.clone();
                let mut href_updated = false;

                // 查找 href 属性
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        let key = attr.key.as_ref();
                        if key == b"href" || key == b"xlink:href" {
                            let value = attr.unescape_value()?;

                            // 跳过已经是 data: 的
                            if !value.starts_with("data:") {
                                // 尝试嵌入图片
                                if let Ok(data_uri) = embed_image_file(&value, project_path) {
                                    // 更新 href 属性
                                    new_elem = update_href_attribute(&e, key, &data_uri)?;
                                    href_updated = true;
                                }
                            }
                        }
                    }
                }

                if href_updated {
                    writer.write_event(Event::Empty(new_elem))?;
                } else {
                    writer.write_event(Event::Empty(e))?;
                }
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"image" => {
                let mut new_elem = e.clone();
                let mut href_updated = false;

                // 查找 href 属性
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        let key = attr.key.as_ref();
                        if key == b"href" || key == b"xlink:href" {
                            let value = attr.unescape_value()?;

                            // 跳过已经是 data: 的
                            if !value.starts_with("data:") {
                                // 尝试嵌入图片
                                if let Ok(data_uri) = embed_image_file(&value, project_path) {
                                    // 更新 href 属性
                                    new_elem = update_href_attribute(&e, key, &data_uri)?;
                                    href_updated = true;
                                }
                            }
                        }
                    }
                }

                if href_updated {
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

/// 嵌入单个图片文件
fn embed_image_file(img_path: &str, project_path: &Path) -> Result<String> {
    // 解码 HTML 实体
    let img_path_decoded = html_escape::decode_html_entities(img_path);

    // 处理相对路径
    let full_path = if Path::new(img_path_decoded.as_ref()).is_absolute() {
        Path::new(img_path_decoded.as_ref()).to_path_buf()
    } else {
        project_path.join(img_path_decoded.as_ref())
    };

    // 读取图片文件
    let img_data = std::fs::read(&full_path)
        .with_context(|| format!("Failed to read image file: {}", full_path.display()))?;

    // Base64 编码
    let b64_data = general_purpose::STANDARD.encode(&img_data);

    // 获取 MIME 类型
    let mime_type = get_mime_type(img_path);

    Ok(format!("data:{};base64,{}", mime_type, b64_data))
}

/// 更新元素的 href 属性
fn update_href_attribute(
    elem: &BytesStart,
    href_key: &[u8],
    new_value: &str,
) -> Result<BytesStart> {
    let mut new_elem = BytesStart::new(std::str::from_utf8(elem.name().as_ref())?);

    for attr in elem.attributes() {
        let attr = attr?;
        if attr.key.as_ref() == href_key {
            new_elem.push_attribute((href_key, new_value));
        } else {
            new_elem.push_attribute(attr);
        }
    }

    Ok(new_elem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type("test.png"), "image/png");
        assert_eq!(get_mime_type("test.jpg"), "image/jpeg");
        assert_eq!(get_mime_type("test.jpeg"), "image/jpeg");
        assert_eq!(get_mime_type("test.gif"), "image/gif");
        assert_eq!(get_mime_type("test.webp"), "image/webp");
    }

    #[test]
    fn test_embed_images_no_images() {
        let temp_dir = TempDir::new().unwrap();
        let input = r#"<svg><rect x="0" y="0" width="100" height="100"/></svg>"#;
        let output = embed_images(input, temp_dir.path()).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_embed_images_with_data_uri() {
        let temp_dir = TempDir::new().unwrap();
        let input = r#"<svg><image href="data:image/png;base64,iVBORw0KGgo="/></svg>"#;
        let output = embed_images(input, temp_dir.path()).unwrap();
        // 应该保持不变
        assert!(output.contains("data:image/png;base64"));
    }

    #[test]
    fn test_embed_images_with_external_file() {
        let temp_dir = TempDir::new().unwrap();
        let img_path = temp_dir.path().join("test.png");

        // 创建一个简单的 PNG 文件（1x1 像素）
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // width=1, height=1
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // rest of IHDR
            0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
            0x08, 0x99, 0x63, 0xF8, 0x0F, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x18,
            0xDD, 0x8D, 0xB4, // IDAT data
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60,
            0x82, // IEND chunk
        ];
        fs::write(&img_path, png_data).unwrap();

        let input = format!(
            r#"<svg><image href="{}"/></svg>"#,
            img_path.file_name().unwrap().to_str().unwrap()
        );
        let output = embed_images(&input, temp_dir.path()).unwrap();

        // 应该包含 data:image/png;base64
        assert!(output.contains("data:image/png;base64"));
        // 不应该包含原始文件名
        assert!(!output.contains("test.png"));
    }
}
