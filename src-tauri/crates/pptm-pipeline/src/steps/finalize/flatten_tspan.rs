// 文本扁平化模块
//
// 将 <tspan> 转为独立 <text>（用于特殊渲染器）

use anyhow::Result;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

/// 文本扁平化
///
/// # Arguments
///
/// * `svg_content` - SVG 文件内容
///
/// # Returns
///
/// 处理后的 SVG 内容
pub fn flatten_tspan(svg_content: &str) -> Result<String> {
    let mut reader = Reader::from_str(svg_content);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut in_text = false;
    let mut text_attrs = Vec::new();
    let mut tspans = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"text" => {
                in_text = true;
                text_attrs.clear();
                tspans.clear();

                // 保存 text 元素的属性
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        text_attrs.push((
                            attr.key.as_ref().to_vec(),
                            attr.value.to_vec(),
                        ));
                    }
                }
            }
            Ok(Event::Start(e)) if in_text && e.name().as_ref() == b"tspan" => {
                // 收集 tspan 信息
                let mut tspan_info = TspanInfo::default();

                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let value = attr.unescape_value().unwrap_or_default();

                        match key {
                            "x" => tspan_info.x = Some(value.to_string()),
                            "y" => tspan_info.y = Some(value.to_string()),
                            "dx" => tspan_info.dx = Some(value.to_string()),
                            "dy" => tspan_info.dy = Some(value.to_string()),
                            "fill" => tspan_info.fill = Some(value.to_string()),
                            "font-size" => tspan_info.font_size = Some(value.to_string()),
                            "font-weight" => tspan_info.font_weight = Some(value.to_string()),
                            "font-family" => tspan_info.font_family = Some(value.to_string()),
                            "style" => tspan_info.style = Some(value.to_string()),
                            _ => {}
                        }
                    }
                }

                tspans.push(tspan_info);
            }
            Ok(Event::Text(e)) if in_text && !tspans.is_empty() => {
                // 保存 tspan 的文本内容
                if let Some(last_tspan) = tspans.last_mut() {
                    last_tspan.text = e.unescape().unwrap_or_default().to_string();
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"text" => {
                in_text = false;

                // 如果有 tspan，将它们转换为独立的 text 元素
                if !tspans.is_empty() {
                    for tspan in &tspans {
                        write_text_element(&mut writer, &text_attrs, tspan)?;
                    }
                } else {
                    // 没有 tspan，保持原样
                    let mut text_elem = BytesStart::new("text");
                    for (key, value) in &text_attrs {
                        text_elem.push_attribute((key.as_slice(), value.as_slice()));
                    }
                    writer.write_event(Event::Start(text_elem))?;
                    writer.write_event(Event::End(e))?;
                }
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                if !in_text {
                    writer.write_event(e)?;
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to parse SVG: {}", e));
            }
        }

        buf.clear();
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

#[derive(Debug, Default)]
struct TspanInfo {
    x: Option<String>,
    y: Option<String>,
    dx: Option<String>,
    dy: Option<String>,
    fill: Option<String>,
    font_size: Option<String>,
    font_weight: Option<String>,
    font_family: Option<String>,
    style: Option<String>,
    text: String,
}

fn write_text_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    text_attrs: &[(Vec<u8>, Vec<u8>)],
    tspan: &TspanInfo,
) -> Result<()> {
    let mut text_elem = BytesStart::new("text");

    // 添加原 text 元素的属性
    for (key, value) in text_attrs {
        let key_str = std::str::from_utf8(key).unwrap_or("");
        // 如果 tspan 有覆盖的属性，跳过
        if matches!(key_str, "x" | "y") && (tspan.x.is_some() || tspan.y.is_some()) {
            continue;
        }
        text_elem.push_attribute((key.as_slice(), value.as_slice()));
    }

    // 添加 tspan 的属性
    if let Some(x) = &tspan.x {
        text_elem.push_attribute(("x", x.as_str()));
    }
    if let Some(y) = &tspan.y {
        text_elem.push_attribute(("y", y.as_str()));
    }
    if let Some(fill) = &tspan.fill {
        text_elem.push_attribute(("fill", fill.as_str()));
    }
    if let Some(font_size) = &tspan.font_size {
        text_elem.push_attribute(("font-size", font_size.as_str()));
    }
    if let Some(font_weight) = &tspan.font_weight {
        text_elem.push_attribute(("font-weight", font_weight.as_str()));
    }
    if let Some(font_family) = &tspan.font_family {
        text_elem.push_attribute(("font-family", font_family.as_str()));
    }
    if let Some(style) = &tspan.style {
        text_elem.push_attribute(("style", style.as_str()));
    }

    writer.write_event(Event::Start(text_elem))?;
    writer.write_event(Event::Text(quick_xml::events::BytesText::new(&tspan.text)))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("text")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_tspan_no_tspan() {
        let input = r#"<svg><text x="100" y="200">Hello</text></svg>"#;
        let output = flatten_tspan(input).unwrap();
        // 应该保持不变
        assert!(output.contains("<text"));
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_flatten_tspan_with_tspan() {
        let input = r#"<svg><text x="100" y="200"><tspan x="100" y="200">Hello</tspan></text></svg>"#;
        let output = flatten_tspan(input).unwrap();
        // 应该转换为独立的 text 元素
        assert!(output.contains("<text"));
        assert!(output.contains("Hello"));
        // 不应该包含 tspan
        assert!(!output.contains("tspan"));
    }
}
