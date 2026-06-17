use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

use crate::model::Spectrum;

/// 光谱数据解析器
pub struct SpectrumParser;

impl SpectrumParser {
    /// 从二进制数据中提取所有光谱数组
    ///
    /// 使用暴力扫描方式，逐字节搜索连续合法的 float32 数据块，
    /// 与 Python 脚本的 extract_float_arrays 行为一致。
    pub fn extract_spectra(data: &[u8]) -> Vec<Spectrum> {
        let mut spectra = Vec::new();
        let mut i = 0;
        let mut pos_index = 0;

        while i + 4 <= data.len() {
            let mut cursor = Cursor::new(&data[i..]);
            if let Ok(val) = cursor.read_f32::<BigEndian>() {
                if val.is_finite() && val.abs() > 0.001 && val.abs() < 10000.0 {
                    let mut values = vec![val];
                    let mut j = i + 4;

                    while j + 4 <= data.len() {
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
        let spectra = SpectrumParser::extract_spectra(&[]);
        assert!(spectra.is_empty());
    }

    #[test]
    fn test_extract_single_block() {
        let mut data = Vec::new();
        for i in 0..601 {
            let val: f32 = 0.5 + i as f32 * 0.001;
            data.extend_from_slice(&val.to_be_bytes());
        }
        let spectra = SpectrumParser::extract_spectra(&data);
        assert_eq!(spectra.len(), 1);
        assert_eq!(spectra[0].values.len(), 601);
    }

    #[test]
    fn test_extract_multiple_blocks() {
        let mut data = Vec::new();
        // 块 1：601 个有效值
        for i in 0..601 {
            let val: f32 = 0.5 + i as f32 * 0.001;
            data.extend_from_slice(&val.to_be_bytes());
        }
        // 间隔（无效数据）
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // 块 2：200 个有效值
        for i in 0..200 {
            let val: f32 = 1.0 + i as f32 * 0.01;
            data.extend_from_slice(&val.to_be_bytes());
        }

        let spectra = SpectrumParser::extract_spectra(&data);
        assert_eq!(spectra.len(), 2);
        assert_eq!(spectra[0].values.len(), 601);
        assert_eq!(spectra[1].values.len(), 200);
    }

    #[test]
    fn test_skip_short_blocks() {
        // 少于 100 个值的块应被跳过
        let mut data = Vec::new();
        for i in 0..50 {
            let val: f32 = 1.0 + i as f32;
            data.extend_from_slice(&val.to_be_bytes());
        }
        let spectra = SpectrumParser::extract_spectra(&data);
        assert!(spectra.is_empty());
    }

    #[test]
    fn test_values_in_valid_range() {
        // NaN 中断数据块：前 100 个值满足阈值，后 99 个不满足
        let mut data = Vec::new();
        for i in 0..200 {
            let val: f32 = if i == 100 { f32::NAN } else { 1.0 + i as f32 * 0.01 };
            data.extend_from_slice(&val.to_be_bytes());
        }
        let spectra = SpectrumParser::extract_spectra(&data);
        assert_eq!(spectra.len(), 1);
        assert_eq!(spectra[0].values.len(), 100);
    }
}
