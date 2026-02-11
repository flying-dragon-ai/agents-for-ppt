// Native OOXML 后端
// 使用 Rust 原生实现生成 OOXML 格式的 PPTX

use crate::{PptxBackend, PptxConfig, Result, Slide, SlideContent};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

/// Native OOXML 后端
///
/// 使用 Rust 原生实现生成 OOXML 格式的 PPTX
pub struct NativeOoxml;

impl NativeOoxml {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativeOoxml {
    fn default() -> Self {
        Self::new()
    }
}

impl PptxBackend for NativeOoxml {
    fn export(&self, slides: &[Slide], output_path: &Path, config: &PptxConfig) -> Result<()> {
        let file = File::create(output_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // 写入 [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(self.generate_content_types(slides.len()).as_bytes())?;

        // 写入 _rels/.rels
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(self.generate_root_rels().as_bytes())?;

        // 写入 ppt/presentation.xml
        zip.start_file("ppt/presentation.xml", options)?;
        zip.write_all(self.generate_presentation(slides.len(), config).as_bytes())?;

        // 写入 ppt/_rels/presentation.xml.rels
        zip.start_file("ppt/_rels/presentation.xml.rels", options)?;
        zip.write_all(self.generate_presentation_rels(slides.len()).as_bytes())?;

        // 写入每个幻灯片
        for (index, slide) in slides.iter().enumerate() {
            let slide_num = index + 1;

            // 写入 ppt/slides/slide{n}.xml
            zip.start_file(format!("ppt/slides/slide{}.xml", slide_num), options)?;
            zip.write_all(self.generate_slide(slide, config).as_bytes())?;

            // 写入 ppt/slides/_rels/slide{n}.xml.rels
            zip.start_file(
                format!("ppt/slides/_rels/slide{}.xml.rels", slide_num),
                options,
            )?;
            zip.write_all(self.generate_slide_rels(slide_num).as_bytes())?;

            // 如果有演讲备注，写入 ppt/notesSlides/notesSlide{n}.xml
            if slide.notes.is_some() {
                zip.start_file(
                    format!("ppt/notesSlides/notesSlide{}.xml", slide_num),
                    options,
                )?;
                zip.write_all(self.generate_notes_slide(slide).as_bytes())?;
            }

            // 如果是 PNG 内容，写入图片文件
            if let SlideContent::Png(png_data) = &slide.content {
                zip.start_file(format!("ppt/media/image{}.png", slide_num), options)?;
                zip.write_all(png_data)?;
            }
        }

        // 写入 ppt/slideLayouts/slideLayout1.xml（简化版）
        zip.start_file("ppt/slideLayouts/slideLayout1.xml", options)?;
        zip.write_all(self.generate_slide_layout().as_bytes())?;

        // 写入 ppt/slideMasters/slideMaster1.xml（简化版）
        zip.start_file("ppt/slideMasters/slideMaster1.xml", options)?;
        zip.write_all(self.generate_slide_master().as_bytes())?;

        zip.finish()?;
        Ok(())
    }

    fn name(&self) -> &str {
        "native_ooxml"
    }

    fn is_available(&self) -> bool {
        true // 原生实现总是可用
    }
}

impl NativeOoxml {
    fn generate_content_types(&self, slide_count: usize) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="png" ContentType="image/png"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
"#,
        );

        for i in 1..=slide_count {
            xml.push_str(&format!(
                r#"  <Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
"#,
                i
            ));
            xml.push_str(&format!(
                r#"  <Override PartName="/ppt/notesSlides/notesSlide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"/>
"#,
                i
            ));
        }

        xml.push_str(r#"  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
</Types>"#);

        xml
    }

    fn generate_root_rels(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#.to_string()
    }

    fn generate_presentation(&self, slide_count: usize, config: &PptxConfig) -> String {
        let mut xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId1"/>
  </p:sldMasterIdLst>
  <p:sldIdLst>
"#.to_string();

        for i in 1..=slide_count {
            xml.push_str(&format!(
                r#"    <p:sldId id="{}" r:id="rId{}"/>
"#,
                255 + i,
                i + 1
            ));
        }

        xml.push_str(&format!(
            r#"  </p:sldIdLst>
  <p:sldSz cx="{}" cy="{}"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
            config.width * 9525,
            config.height * 9525
        ));

        xml
    }

    fn generate_presentation_rels(&self, slide_count: usize) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
"#,
        );

        for i in 1..=slide_count {
            xml.push_str(&format!(
                r#"  <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>
"#,
                i + 1,
                i
            ));
        }

        xml.push_str("</Relationships>");
        xml
    }

    fn generate_slide(&self, slide: &Slide, config: &PptxConfig) -> String {
        let content = match &slide.content {
            SlideContent::Svg(svg) => self.svg_to_pml(svg, config),
            SlideContent::Png(_) => self.png_to_pml(slide.number, config),
        };

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm>
          <a:off x="0" y="0"/>
          <a:ext cx="{}" cy="{}"/>
          <a:chOff x="0" y="0"/>
          <a:chExt cx="{}" cy="{}"/>
        </a:xfrm>
      </p:grpSpPr>
      {}
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr>
    <a:masterClrMapping/>
  </p:clrMapOvr>
</p:sld>"#,
            config.width * 9525,
            config.height * 9525,
            config.width * 9525,
            config.height * 9525,
            content
        )
    }

    fn svg_to_pml(&self, _svg: &str, _config: &PptxConfig) -> String {
        // TODO: 实现 SVG 到 PresentationML 的转换
        // 这是一个复杂的过程，需要解析 SVG 并转换为 OOXML 图形元素
        // 目前返回一个占位符文本
        r#"<p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="SVG Content"/>
          <p:cNvSpPr/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="914400" y="914400"/>
            <a:ext cx="9144000" cy="5486400"/>
          </a:xfrm>
          <a:prstGeom prst="rect">
            <a:avLst/>
          </a:prstGeom>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p>
            <a:r>
              <a:rPr lang="zh-CN"/>
              <a:t>SVG Content (TODO: Implement SVG to PML conversion)</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>"#
            .to_string()
    }

    fn png_to_pml(&self, slide_num: usize, config: &PptxConfig) -> String {
        format!(
            r#"<p:pic>
        <p:nvPicPr>
          <p:cNvPr id="2" name="Image {}"/>
          <p:cNvPicPr>
            <a:picLocks noChangeAspect="1"/>
          </p:cNvPicPr>
          <p:nvPr/>
        </p:nvPicPr>
        <p:blipFill>
          <a:blip r:embed="rId1"/>
          <a:stretch>
            <a:fillRect/>
          </a:stretch>
        </p:blipFill>
        <p:spPr>
          <a:xfrm>
            <a:off x="0" y="0"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="rect">
            <a:avLst/>
          </a:prstGeom>
        </p:spPr>
      </p:pic>"#,
            slide_num,
            config.width * 9525,
            config.height * 9525
        )
    }

    fn generate_slide_rels(&self, slide_num: usize) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image{}.png"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide{}.xml"/>
</Relationships>"#,
            slide_num, slide_num
        )
    }

    fn generate_notes_slide(&self, slide: &Slide) -> String {
        let notes_text = slide.notes.as_deref().unwrap_or("");

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm>
          <a:off x="0" y="0"/>
          <a:ext cx="6858000" cy="9144000"/>
          <a:chOff x="0" y="0"/>
          <a:chExt cx="6858000" cy="9144000"/>
        </a:xfrm>
      </p:grpSpPr>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Notes"/>
          <p:cNvSpPr/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p>
            <a:r>
              <a:rPr lang="zh-CN"/>
              <a:t>{}</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr>
    <a:masterClrMapping/>
  </p:clrMapOvr>
</p:notes>"#,
            escape_xml(notes_text)
        )
    }

    fn generate_slide_layout(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank" preserve="1">
  <p:cSld name="Blank">
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr/>
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr>
    <a:masterClrMapping/>
  </p:clrMapOvr>
</p:sldLayout>"#
            .to_string()
    }

    fn generate_slide_master(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr/>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
  <p:sldLayoutIdLst>
    <p:sldLayoutId id="2147483649" r:id="rId1"/>
  </p:sldLayoutIdLst>
</p:sldMaster>"#
            .to_string()
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
