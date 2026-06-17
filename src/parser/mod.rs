pub mod metadata;
pub mod spectrum;
pub mod tlv;

use crate::error::AppError;
use crate::model::StunnerReport;
use crate::parser::metadata::MetadataParser;
use crate::parser::spectrum::SpectrumParser;
use std::fs;
use std::path::Path;

/// 文件解析器 trait
pub trait FileParser {
    /// 从文件路径解析
    fn parse_file(&self, path: &Path) -> Result<StunnerReport, AppError>;

    /// 从内存字节解析
    fn parse_bytes(&self, data: &[u8], source: &Path) -> Result<StunnerReport, AppError>;
}

/// Stunner bin 文件解析器
pub struct StunnerParser;

impl StunnerParser {
    /// 创建新的解析器实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for StunnerParser {
    fn default() -> Self {
        Self::new()
    }
}

impl FileParser for StunnerParser {
    fn parse_file(&self, path: &Path) -> Result<StunnerReport, AppError> {
        let data = fs::read(path).map_err(|e| AppError::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;
        self.parse_bytes(&data, path)
    }

    fn parse_bytes(&self, data: &[u8], source: &Path) -> Result<StunnerReport, AppError> {
        // 1. 解析元数据
        let mut report = MetadataParser::parse(data, source)?;

        // 2. 提取光谱数据
        report.spectra = SpectrumParser::extract_spectra(data);

        // 3. 提取吸光度数据
        Self::extract_absorbance_data(&mut report, data);

        Ok(report)
    }
}

impl StunnerParser {
    /// 提取吸光度数据
    fn extract_absorbance_data(report: &mut StunnerReport, data: &[u8]) {
        // 查找 absorbance_threshold 标记
        if let Some(offset) = Self::find_marker(data, b"absorbance_threshold") {
            if let Some(values) = Self::read_float_array_at(data, offset) {
                report.absorbance.thresholds = values;
            }
        }

        // 查找 absorbance_transition 标记
        if let Some(offset) = Self::find_marker(data, b"absorbance_transition") {
            if let Some(values) = Self::read_float_array_at(data, offset) {
                report.absorbance.transitions = values;
            }
        }
    }

    /// 查找标记位置
    fn find_marker(data: &[u8], marker: &[u8]) -> Option<usize> {
        data.windows(marker.len())
            .position(|window| window == marker)
            .map(|pos| pos + marker.len())
    }

    /// 在指定位置读取浮点数组
    fn read_float_array_at(data: &[u8], offset: usize) -> Option<Vec<f32>> {
        use byteorder::{BigEndian, ReadBytesExt};
        use std::io::Cursor;

        if offset + 4 > data.len() {
            return None;
        }

        let mut cursor = Cursor::new(&data[offset..]);
        let len = cursor.read_u32::<BigEndian>().ok()? as usize;

        if offset + 4 + len > data.len() || len < 4 {
            return None;
        }

        let count = len / 4;
        let mut cursor = Cursor::new(&data[offset + 4..offset + 4 + len]);
        let mut result = Vec::with_capacity(count);

        for _ in 0..count {
            if let Ok(val) = cursor.read_f32::<BigEndian>() {
                result.push(val);
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stunner_parser_default() {
        let parser = StunnerParser::new();
        // 确保可以创建解析器
        let data = &[];
        let path = Path::new("test.bin");
        let report = parser.parse_bytes(data, path).unwrap();
        assert!(report.spectra.is_empty());
    }

    #[test]
    fn test_find_marker() {
        let data = b"some data absorbance_threshold here";
        let pos = StunnerParser::find_marker(data, b"absorbance_threshold");
        assert!(pos.is_some());
    }

    #[test]
    fn test_find_marker_not_found() {
        let data = b"some data without marker";
        let pos = StunnerParser::find_marker(data, b"absorbance_threshold");
        assert!(pos.is_none());
    }
}
