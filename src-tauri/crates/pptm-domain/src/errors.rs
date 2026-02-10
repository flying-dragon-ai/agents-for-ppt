use thiserror::Error;

#[derive(Error, Debug)]
pub enum PptmError {
    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("项目初始化失败: {0}")]
    ProjectInitError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}
