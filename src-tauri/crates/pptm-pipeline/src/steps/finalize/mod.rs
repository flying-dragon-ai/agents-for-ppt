use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// SVG 后处理选项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeOptions {
    pub embed_icons: bool,
    pub crop_images: bool,
    pub fix_aspect: bool,
    pub embed_images: bool,
    pub flatten_text: bool,
    pub fix_rounded: bool,
}

impl Default for FinalizeOptions {
    fn default() -> Self {
        Self {
            embed_icons: true,
            crop_images: true,
            fix_aspect: true,
            embed_images: true,
            flatten_text: true,
            fix_rounded: true,
        }
    }
}

/// 运行项目级 SVG 后处理。
///
/// 当前实现为稳定可用的基线版本：
/// 1. 读取 `svg_output/`
/// 2. 复制 SVG 到 `svg_final/`
/// 3. 为后续细粒度处理步骤预留统一入口
pub fn finalize_project(project_path: &Path, _options: &FinalizeOptions) -> Result<()> {
    let svg_output = project_path.join("svg_output");
    let svg_final = project_path.join("svg_final");

    if !svg_output.exists() {
        anyhow::bail!("缺少 svg_output 目录: {}", svg_output.display());
    }

    fs::create_dir_all(&svg_final)
        .context(format!("创建 svg_final 目录失败: {}", svg_final.display()))?;

    for entry in fs::read_dir(&svg_output).context(format!(
        "读取 svg_output 目录失败: {}",
        svg_output.display()
    ))? {
        let entry = entry.context("读取目录项失败")?;
        let path = entry.path();

        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
        {
            let target = svg_final.join(path.file_name().expect("SVG 文件应有文件名"));

            fs::copy(&path, &target).context(format!(
                "复制 SVG 失败: {} -> {}",
                path.display(),
                target.display()
            ))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finalize_project_copies_svgs() {
        let temp_dir = tempfile::tempdir().expect("应能创建临时目录");
        let project_path = temp_dir.path();
        let svg_output = project_path.join("svg_output");

        fs::create_dir_all(&svg_output).expect("应能创建 svg_output");
        fs::write(svg_output.join("01_封面.svg"), "<svg></svg>").expect("应能写入测试 SVG");
        fs::write(svg_output.join("readme.txt"), "not svg").expect("应能写入测试文本");

        finalize_project(project_path, &FinalizeOptions::default()).expect("后处理应成功");

        let svg_final = project_path.join("svg_final");
        assert!(svg_final.join("01_封面.svg").exists());
        assert!(!svg_final.join("readme.txt").exists());
    }
}
