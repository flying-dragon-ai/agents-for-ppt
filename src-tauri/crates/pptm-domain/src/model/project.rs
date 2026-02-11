use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 项目运行时信息（用于命令返回与界面展示）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// 项目绝对路径。
    pub path: PathBuf,
    /// 项目名称。
    pub name: String,
    /// 画布格式键，例如 `ppt169`。
    pub format: String,
    /// 画布格式显示名，例如 `PPT 16:9`。
    pub format_name: String,
    /// 项目创建时间（字符串格式）。
    pub created_at: String,
    /// 当前 SVG 页数统计。
    pub svg_count: usize,
    /// 是否存在设计规范文件。
    pub has_spec: bool,
}

/// 项目持久化元数据（用于 `.pptm-meta.json`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// 元数据版本号。
    pub version: String,
    /// 项目名称。
    pub name: String,
    /// 画布格式键。
    pub format: String,
    /// 创建时间。
    pub created_at: String,
    /// 更新时间。
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_info_serde_roundtrip() {
        let info = ProjectInfo {
            path: PathBuf::from("D:/projects/demo_ppt169_20260210"),
            name: "demo".to_string(),
            format: "ppt169".to_string(),
            format_name: "PPT 16:9".to_string(),
            created_at: "2026-02-10".to_string(),
            svg_count: 12,
            has_spec: true,
        };

        let json = serde_json::to_string(&info).expect("序列化 ProjectInfo 应成功");
        let decoded: ProjectInfo =
            serde_json::from_str(&json).expect("反序列化 ProjectInfo 应成功");

        assert_eq!(decoded, info);
    }

    #[test]
    fn test_project_metadata_serde_roundtrip() {
        let metadata = ProjectMetadata {
            version: "1.0.0".to_string(),
            name: "demo".to_string(),
            format: "ppt169".to_string(),
            created_at: "2026-02-10T00:00:00Z".to_string(),
            updated_at: "2026-02-10T12:30:00Z".to_string(),
        };

        let json = serde_json::to_string(&metadata).expect("序列化 ProjectMetadata 应成功");
        let decoded: ProjectMetadata =
            serde_json::from_str(&json).expect("反序列化 ProjectMetadata 应成功");

        assert_eq!(decoded, metadata);
    }
}
