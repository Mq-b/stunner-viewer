use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

use crate::error::AppError;
use crate::model::Spectrum;

/// 光谱数据解析器
pub struct SpectrumParser;

/// 光谱数据标记关键字
const SPECTRUM_MARKERS: &[&[u8]] = &[
    b"spectrum_measurement",
    b"absorbance_threshold",
    b"absorbance_transition",
    b"dark_measurement",
    b"freespace_measurement",
];

impl SpectrumParser {
    /// 从二进制数据中提取所有光谱数组
    pub fn extract_spectra(data: &[u8]) -> Vec<Spectrum> {
        let mut spectra = Vec::new();
        let mut pos_index = 0;

        // 方法1: 按标记关键字定位
        for marker in SPECTRUM_MARKERS {
            let positions = Self::find_marker_positions(data, marker);
            for offset in positions {
                if let Some(spectrum) = Self::try_parse_at_offset(data, offset, pos_index) {
                    spectra.push(spectrum);
                    pos_index += 1;
                }
            }
        }

        // 方法2: 如果标记法未找到足够数据，使用暴力扫描
        if spectra.is_empty() {
            spectra = Self::brute_force_extract(data);
        }

        spectra
    }

    /// 查找标记在数据中的位置
    fn find_marker_positions(data: &[u8], marker: &[u8]) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut start = 0;

        while start + marker.len() < data.len() {
            if let Some(pos) = Self::find_subsequence(&data[start..], marker) {
                let abs_pos = start + pos + marker.len();
                positions.push(abs_pos);
                start = abs_pos;
            } else {
                break;
            }
        }

        positions
    }

    /// 查找子序列
    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }

    /// 尝试在指定偏移处解析浮点数组
    fn try_parse_at_offset(data: &[u8], offset: usize, pos_index: usize) -> Option<Spectrum> {
        if offset + 4 > data.len() {
            return None;
        }

        // 读取长度前缀
        let len = {
            let mut cursor = Cursor::new(&data[offset..]);
            cursor.read_u32::<BigEndian>().ok()? as usize
        };

        // 验证长度合理性
        if len < 100 || len > 100_000 || offset + 4 + len > data.len() {
            return None;
        }

        // 尝试解析为 float32 数组
        let float_count = len / 4;
        if float_count < 50 {
            return None;
        }

        let values = Self::read_float32_array(&data[offset + 4..offset + 4 + len], float_count);

        // 验证数据有效性
        if !Self::validate_spectrum_data(&values) {
            return None;
        }

        Some(Spectrum::new(pos_index, values))
    }

    /// 读取 float32 数组
    fn read_float32_array(data: &[u8], count: usize) -> Vec<f32> {
        let mut cursor = Cursor::new(data);
        let mut result = Vec::with_capacity(count);

        for _ in 0..count {
            if let Ok(val) = cursor.read_f32::<BigEndian>() {
                result.push(val);
            } else {
                break;
            }
        }

        result
    }

    /// 验证光谱数据有效性
    fn validate_spectrum_data(values: &[f32]) -> bool {
        if values.len() < 50 {
            return false;
        }

        // 检查是否有有效值（非 NaN、非无穷）
        let valid_count = values
            .iter()
            .filter(|v| v.is_finite() && v.abs() > 0.001 && v.abs() < 10000.0)
            .count();

        // 至少 80% 的值应该是有效的
        valid_count as f64 / values.len() as f64 > 0.8
    }

    /// 暴力扫描提取浮点数组
    fn brute_force_extract(data: &[u8]) -> Vec<Spectrum> {
        let mut spectra = Vec::new();
        let mut i = 0;
        let mut pos_index = 0;

        while i + 4 <= data.len() {
            // 尝试读取一个 float32
            let mut cursor = Cursor::new(&data[i..]);
            if let Ok(val) = cursor.read_f32::<BigEndian>() {
                if val.is_finite() && val.abs() > 0.001 && val.abs() < 10000.0 {
                    // 找到一个有效浮点数，尝试读取连续的浮点数组
                    let mut values = vec![val];
                    let mut j = i + 4;

                    while j + 4 <= data.len() && values.len() < 1000 {
                        let mut cursor = Cursor::new(&data[j..]);
                        if let Ok(v) = cursor.read_f32::<BigEndian>() {
                            if v.is_finite() && v.abs() > 0.001 && v.abs() < 10000.0 {
                                values.push(v);
                                j += 4;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    // 如果找到足够多的连续浮点数，认为是一个光谱数据块
                    if values.len() >= 100 {
                        spectra.push(Spectrum::new(pos_index, values));
                        pos_index += 1;
                        i = j;
                        continue;
                    }
                }
            }
            i += 1;
        }

        spectra
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_empty_data() {
        let data = &[];
        let spectra = SpectrumParser::extract_spectra(data);
        assert!(spectra.is_empty());
    }

    #[test]
    fn test_validate_spectrum_data() {
        // 有效数据
        let valid: Vec<f32> = (0..100).map(|i| 1.0 + i as f32 * 0.01).collect();
        assert!(SpectrumParser::validate_spectrum_data(&valid));

        // 无效数据（太少）
        let too_short: Vec<f32> = vec![1.0; 10];
        assert!(!SpectrumParser::validate_spectrum_data(&too_short));

        // 无效数据（包含大量 NaN - 超过 20%）
        let mut with_nan: Vec<f32> = vec![1.0; 100];
        for i in 0..30 {
            with_nan[i] = f32::NAN;
        }
        assert!(!SpectrumParser::validate_spectrum_data(&with_nan));
    }

    #[test]
    fn test_brute_force_extract() {
        // 构造测试数据：601个有效浮点数
        let mut data = Vec::new();
        for i in 0..601 {
            let val: f32 = 100.0 + i as f32;
            data.extend_from_slice(&val.to_be_bytes());
        }

        let spectra = SpectrumParser::brute_force_extract(&data);
        assert_eq!(spectra.len(), 1);
        assert_eq!(spectra[0].len(), 601);
    }

    #[test]
    fn test_find_marker_positions() {
        let mut data = Vec::new();
        data.extend_from_slice(b"some data");
        data.extend_from_slice(b"spectrum_measurement");
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0C]); // length = 12
        data.extend_from_slice(&[0x3F, 0x80, 0x00, 0x00]); // 1.0f32
        data.extend_from_slice(&[0x40, 0x00, 0x00, 0x00]); // 2.0f32
        data.extend_from_slice(&[0x40, 0x40, 0x00, 0x00]); // 3.0f32

        let positions = SpectrumParser::find_marker_positions(&data, b"spectrum_measurement");
        assert_eq!(positions.len(), 1);
    }

    #[test]
    fn test_read_float32_array() {
        let mut data = Vec::new();
        for i in 0..5 {
            let val: f32 = 1.0 + i as f32;
            data.extend_from_slice(&val.to_be_bytes());
        }

        let result = SpectrumParser::read_float32_array(&data, 5);
        assert_eq!(result.len(), 5);
        assert!((result[0] - 1.0).abs() < 1e-6);
        assert!((result[4] - 5.0).abs() < 1e-6);
    }
}
