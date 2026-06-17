use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

use crate::error::AppError;

/// TLV 底层值类型
#[derive(Debug, Clone)]
pub enum TlvValue {
    /// UTF-8 字符串
    String(String),
    /// 无符号整数
    Uint(u64),
    /// 32 位浮点
    Float32(f32),
    /// 64 位浮点
    Float64(f64),
    /// 布尔值
    Bool(bool),
    /// 原始字节数组
    Raw(Vec<u8>),
}

/// 单个 TLV 条目
#[derive(Debug, Clone)]
pub struct TlvEntry {
    pub key: String,
    pub value: TlvValue,
}

/// TLV 读取器
pub struct TlvReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> TlvReader<'a> {
    /// 创建新的 TLV 读取器
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// 当前偏移量
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// 剩余字节数
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    /// 是否还有数据
    pub fn has_data(&self) -> bool {
        self.offset < self.data.len()
    }

    /// 读取 u32 (大端序)
    fn read_u32(&mut self) -> Result<u32, AppError> {
        let remaining = self.remaining();
        if remaining < 4 {
            return Err(AppError::TruncatedData {
                expected: 4,
                remaining,
            });
        }
        let mut cursor = Cursor::new(&self.data[self.offset..]);
        let val = cursor
            .read_u32::<BigEndian>()
            .map_err(|_| AppError::TruncatedData {
                expected: 4,
                remaining,
            })?;
        self.offset += 4;
        Ok(val)
    }

    /// 读取指定长度的字节
    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], AppError> {
        let remaining = self.remaining();
        if remaining < len {
            return Err(AppError::TruncatedData {
                expected: len,
                remaining,
            });
        }
        let start = self.offset;
        self.offset += len;
        Ok(&self.data[start..self.offset])
    }

    /// 读取一个 TLV 字符串值
    pub fn read_string(&mut self) -> Result<String, AppError> {
        let len = self.read_u32()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| AppError::ParseError {
            offset: self.offset - len,
            message: format!("无效的 UTF-8 字符串: {}", e),
        })
    }

    /// 读取一个 TLV 整数值（自动判断字节长度）
    pub fn read_int(&mut self) -> Result<u64, AppError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        let mut cursor = Cursor::new(bytes);

        match len {
            1 => Ok(cursor.read_u8().map_err(|_| AppError::TruncatedData {
                expected: 1,
                remaining: 0,
            })? as u64),
            2 => Ok(cursor
                .read_u16::<BigEndian>()
                .map_err(|_| AppError::TruncatedData {
                    expected: 2,
                    remaining: 0,
                })? as u64),
            4 => Ok(cursor
                .read_u32::<BigEndian>()
                .map_err(|_| AppError::TruncatedData {
                    expected: 4,
                    remaining: 0,
                })? as u64),
            8 => Ok(cursor
                .read_u64::<BigEndian>()
                .map_err(|_| AppError::TruncatedData {
                    expected: 8,
                    remaining: 0,
                })?),
            _ => Err(AppError::ParseError {
                offset: self.offset - len,
                message: format!("不支持的整数长度: {} 字节", len),
            }),
        }
    }

    /// 读取一个 TLV 浮点值（自动判断精度）
    pub fn read_float(&mut self) -> Result<f64, AppError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        let mut cursor = Cursor::new(bytes);

        match len {
            4 => Ok(cursor
                .read_f32::<BigEndian>()
                .map_err(|_| AppError::TruncatedData {
                    expected: 4,
                    remaining: 0,
                })? as f64),
            8 => Ok(cursor
                .read_f64::<BigEndian>()
                .map_err(|_| AppError::TruncatedData {
                    expected: 8,
                    remaining: 0,
                })?),
            _ => Err(AppError::ParseError {
                offset: self.offset - len,
                message: format!("不支持的浮点长度: {} 字节", len),
            }),
        }
    }

    /// 读取一个 TLV 布尔值
    pub fn read_bool(&mut self) -> Result<bool, AppError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(bytes.first().map_or(false, |&b| b != 0))
    }

    /// 读取一个完整的 TLV 条目（键值对）
    pub fn read_entry(&mut self) -> Result<TlvEntry, AppError> {
        let key = self.read_string()?;
        let value_offset = self.offset;

        // 尝试根据已知字段名判断值类型
        let value = match key.as_str() {
            // 字符串字段
            "experiment_name" | "experiment_id" | "instrument_S/N" | "instrument_MAC"
            | "software_type" | "software_version" | "control_sw_version"
            | "viewer_sw_version" | "firmware" | "assembly_ID" | "SPM" | "sample_type"
            | "name" | "name_barcode" | "layout_GUID" | "user_GUID" | "chip_id_code"
            | "measurement_position_name" | "source_plate_id" | "password"
            | "disposable_type" | "protocol_GUID" => TlvValue::String(self.read_string()?),

            // 整数字段
            "version" | "nr_of_acquisitions" | "Layout_size"
            | "measurement_positions_size" | "measurement_position"
            | "source_plate_position" | "level" | "disposables_size"
            | "path_length_size" | "measurement_size" | "Profile_measurements_size"
            | "profile_size1" | "profile_size2" | "samples_size" | "blanks_size"
            | "references_size" | "channels_size" | "channel" | "chip_id_index"
            | "disposable_type_id" => TlvValue::Uint(self.read_int()?),

            // 浮点字段
            "nominal_path_length1" | "nominal_path_length2" | "OD_limit1" | "OD_limit2"
            | "path_length1" | "path_length2" | "measurement_position_x"
            | "measurement_position_y" => TlvValue::Float64(self.read_float()?),

            // 布尔字段
            "processed" | "pre-pump?" | "PV_activated?" => TlvValue::Bool(self.read_bool()?),

            // 时间戳字段 (8字节 FILETIME)
            "date" | "calibration date" | "acquisition_time" => {
                let val = self.read_int()?;
                TlvValue::Uint(val)
            }

            // 未知字段 - 尝试自动检测
            _ => self.auto_decode_value()?,
        };

        Ok(TlvEntry { key, value })
    }

    /// 自动检测值类型
    fn auto_decode_value(&mut self) -> Result<TlvValue, AppError> {
        let value_start = self.offset;
        let len = self.read_u32()? as usize;

        // 大数据块
        if len > 100_000 {
            let bytes = self.read_bytes(len)?;
            return Ok(TlvValue::Raw(bytes.to_vec()));
        }

        let bytes = self.read_bytes(len)?;

        match len {
            0 => Ok(TlvValue::String(String::new())),
            1 => Ok(TlvValue::Uint(bytes[0] as u64)),
            2 => {
                let mut cursor = Cursor::new(bytes);
                Ok(TlvValue::Uint(
                    cursor.read_u16::<BigEndian>().unwrap() as u64
                ))
            }
            4 => {
                let mut cursor = Cursor::new(bytes);
                // 尝试浮点
                if let Ok(f) = cursor.read_f32::<BigEndian>() {
                    if f.is_finite() && f.abs() > 0.001 && f.abs() < 1e10 {
                        return Ok(TlvValue::Float32(f));
                    }
                }
                // 回退到整数
                let mut cursor = Cursor::new(bytes);
                Ok(TlvValue::Uint(
                    cursor.read_u32::<BigEndian>().unwrap() as u64
                ))
            }
            8 => {
                let mut cursor = Cursor::new(bytes);
                // 尝试浮点
                if let Ok(f) = cursor.read_f64::<BigEndian>() {
                    if f.is_finite() && f.abs() > 0.001 && f.abs() < 1e15 {
                        return Ok(TlvValue::Float64(f));
                    }
                }
                // 回退到整数
                let mut cursor = Cursor::new(bytes);
                Ok(TlvValue::Uint(cursor.read_u64::<BigEndian>().unwrap()))
            }
            _ => {
                // 尝试作为字符串
                if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                    if s.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                        return Ok(TlvValue::String(s));
                    }
                }
                Ok(TlvValue::Raw(bytes.to_vec()))
            }
        }
    }

    /// 读取 float32 数组（用于光谱数据）
    pub fn read_float32_array(&mut self) -> Result<Vec<f32>, AppError> {
        let len = self.read_u32()? as usize;
        let count = len / 4;
        let bytes = self.read_bytes(len)?;
        let mut cursor = Cursor::new(bytes);
        let mut result = Vec::with_capacity(count);

        for _ in 0..count {
            result.push(
                cursor
                    .read_f32::<BigEndian>()
                    .map_err(|_| AppError::TruncatedData {
                        expected: 4,
                        remaining: 0,
                    })?,
            );
        }

        Ok(result)
    }

    /// 跳过当前值（用于跳过未知字段）
    pub fn skip_value(&mut self) -> Result<(), AppError> {
        let len = self.read_u32()? as usize;
        let remaining = self.remaining();
        if remaining < len {
            return Err(AppError::TruncatedData {
                expected: len,
                remaining,
            });
        }
        self.offset += len;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_string() {
        // 构造: [4字节长度][字符串字节]
        let data = b"\x00\x00\x00\x05hello";
        let mut reader = TlvReader::new(data);
        let s = reader.read_string().unwrap();
        assert_eq!(s, "hello");
        assert_eq!(reader.offset(), 9);
    }

    #[test]
    fn test_read_empty_string() {
        let data = b"\x00\x00\x00\x00";
        let mut reader = TlvReader::new(data);
        let s = reader.read_string().unwrap();
        assert_eq!(s, "");
    }

    #[test]
    fn test_read_u32() {
        let data = b"\x00\x00\x00\x04\x00\x00\x01\x00";
        let mut reader = TlvReader::new(data);
        let val = reader.read_int().unwrap();
        assert_eq!(val, 256);
    }

    #[test]
    fn test_read_u16() {
        let data = b"\x00\x00\x00\x02\x01\x00";
        let mut reader = TlvReader::new(data);
        let val = reader.read_int().unwrap();
        assert_eq!(val, 256);
    }

    #[test]
    fn test_read_u8() {
        let data = b"\x00\x00\x00\x01\x0A";
        let mut reader = TlvReader::new(data);
        let val = reader.read_int().unwrap();
        assert_eq!(val, 10);
    }

    #[test]
    fn test_read_float32() {
        // 1.0f32 大端序 = 0x3F800000
        let data = b"\x00\x00\x00\x04\x3F\x80\x00\x00";
        let mut reader = TlvReader::new(data);
        let val = reader.read_float().unwrap();
        assert!((val - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_read_float64() {
        // 1.0f64 大端序 = 0x3FF0000000000000
        let data = b"\x00\x00\x00\x08\x3F\xF0\x00\x00\x00\x00\x00\x00";
        let mut reader = TlvReader::new(data);
        let val = reader.read_float().unwrap();
        assert!((val - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_read_bool_true() {
        let data = b"\x00\x00\x00\x01\x01";
        let mut reader = TlvReader::new(data);
        assert!(reader.read_bool().unwrap());
    }

    #[test]
    fn test_read_bool_false() {
        let data = b"\x00\x00\x00\x01\x00";
        let mut reader = TlvReader::new(data);
        assert!(!reader.read_bool().unwrap());
    }

    #[test]
    fn test_read_float32_array() {
        // 3个float32: [1.0, 2.0, 3.0]
        let data = b"\x00\x00\x00\x0C\x3F\x80\x00\x00\x40\x00\x00\x00\x40\x40\x00\x00";
        let mut reader = TlvReader::new(data);
        let arr = reader.read_float32_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert!((arr[0] - 1.0).abs() < 1e-6);
        assert!((arr[1] - 2.0).abs() < 1e-6);
        assert!((arr[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_truncated_data() {
        let data = b"\x00\x00\x00\x05hel"; // 声明5字节，实际3字节
        let mut reader = TlvReader::new(data);
        assert!(reader.read_string().is_err());
    }

    #[test]
    fn test_read_entry_known_string_field() {
        // "name" + 值 "test"
        let mut data = Vec::new();
        // key: "name"
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // key长度4
        data.extend_from_slice(b"name");
        // value: "test"
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // value长度4
        data.extend_from_slice(b"test");

        let mut reader = TlvReader::new(&data);
        let entry = reader.read_entry().unwrap();
        assert_eq!(entry.key, "name");
        match entry.value {
            TlvValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("期望字符串值"),
        }
    }

    #[test]
    fn test_read_entry_known_int_field() {
        // "version" + 值 42
        let mut data = Vec::new();
        // key: "version"
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x07]); // key长度7
        data.extend_from_slice(b"version");
        // value: 42 (4字节)
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // value长度4
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x2A]); // 42

        let mut reader = TlvReader::new(&data);
        let entry = reader.read_entry().unwrap();
        assert_eq!(entry.key, "version");
        match entry.value {
            TlvValue::Uint(v) => assert_eq!(v, 42),
            _ => panic!("期望整数值"),
        }
    }

    #[test]
    fn test_multiple_entries() {
        let mut data = Vec::new();
        // entry1: "name" = "test"
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        data.extend_from_slice(b"name");
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        data.extend_from_slice(b"test");
        // entry2: "version" = 1
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x07]);
        data.extend_from_slice(b"version");
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
        data.extend_from_slice(&[0x01]);

        let mut reader = TlvReader::new(&data);

        let e1 = reader.read_entry().unwrap();
        assert_eq!(e1.key, "name");

        let e2 = reader.read_entry().unwrap();
        assert_eq!(e2.key, "version");

        assert!(!reader.has_data());
    }
}
