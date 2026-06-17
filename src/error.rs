use std::path::PathBuf;
use thiserror::Error;

/// 应用统一错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("文件读取失败: {path}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("TLV 解析错误: 偏移 {offset:#X}, {message}")]
    ParseError { offset: usize, message: String },

    #[error("数据截断: 期望 {expected} 字节, 剩余 {remaining} 字节")]
    TruncatedData {
        expected: usize,
        remaining: usize,
    },

    #[error("无效的浮点值")]
    InvalidFloat,

    #[error("XLSX 导出失败: {0}")]
    ExportError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),
}
