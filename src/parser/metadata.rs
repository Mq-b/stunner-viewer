use crate::error::AppError;
use crate::model::{ExperimentInfo, InstrumentInfo, LayoutInfo, MeasurementPosition, StunnerReport};
use crate::parser::tlv::TlvValue;
use std::collections::HashMap;
use std::path::Path;

/// 元数据解析器
///
/// 使用 regex 扫描方式从 bin 二进制数据中提取元数据字段，
/// 兼容 Stunner 非标准 TLV 格式。
pub struct MetadataParser;

/// 检查 pos 位置是否为合法的 TLV 字段匹配
///
/// 验证条件：
/// 1. 字段名前面不能是字母或下划线（避免匹配子串，如 "name" 匹配 "experiment_name"）
/// 2. 字段名后面跟的 4 字节必须是合法的 TLV 长度（< 100000）
fn is_valid_field_match(data: &[u8], pos: usize, field_len: usize) -> bool {
    // 短字段名需要检查前边界
    if field_len <= 8 && pos > 0 {
        let prev = data[pos - 1];
        if prev.is_ascii_alphabetic() || prev == b'_' {
            return false;
        }
    }
    // 检查后面跟的是否为合法 TLV 长度
    let val_start = pos + field_len;
    if val_start + 4 > data.len() {
        return false;
    }
    let val_len = u32::from_be_bytes(data[val_start..val_start + 4].try_into().unwrap()) as usize;
    val_len < 100_000
}

/// 从二进制数据中提取字符串字段值
///
/// 模式：字段名 + 4字节长度 + 可打印ASCII字符串
/// 使用单词边界匹配，避免 "name" 匹配到 "experiment_name"
fn extract_string(data: &[u8], field: &str) -> Option<String> {
    let field_bytes = field.as_bytes();
    let mut pos = 0;
    while pos + field_bytes.len() + 4 <= data.len() {
        if &data[pos..pos + field_bytes.len()] == field_bytes
            && is_valid_field_match(data, pos, field_bytes.len())
        {
            let val_start = pos + field_bytes.len();
            let val_len =
                u32::from_be_bytes(data[val_start..val_start + 4].try_into().ok()?) as usize;
            if val_len > 0
                && val_len <= 500
                && val_start + 4 + val_len <= data.len()
            {
                let val_bytes = &data[val_start + 4..val_start + 4 + val_len];
                // 验证是否为可打印 ASCII
                if val_bytes
                    .iter()
                    .all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
                {
                    let s = String::from_utf8_lossy(val_bytes)
                        .trim_end_matches('\0')
                        .to_string();
                    if !s.is_empty() {
                        return Some(s);
                    }
                }
            }
        }
        pos += 1;
    }
    None
}

/// 从二进制数据中提取整数字段值
fn extract_uint(data: &[u8], field: &str) -> Option<u64> {
    let field_bytes = field.as_bytes();
    let mut pos = 0;
    while pos + field_bytes.len() + 4 <= data.len() {
        if &data[pos..pos + field_bytes.len()] == field_bytes
            && is_valid_field_match(data, pos, field_bytes.len())
        {
            let val_start = pos + field_bytes.len();
            let val_len =
                u32::from_be_bytes(data[val_start..val_start + 4].try_into().ok()?) as usize;
            if val_len > 0 && val_len <= 8 && val_start + 4 + val_len <= data.len() {
                let val_bytes = &data[val_start + 4..val_start + 4 + val_len];
                let mut v: u64 = 0;
                for &b in val_bytes {
                    v = (v << 8) | b as u64;
                }
                return Some(v);
            }
        }
        pos += 1;
    }
    None
}

/// 从二进制数据中提取所有同名字符串字段值
fn extract_all_strings(data: &[u8], field: &str) -> Vec<String> {
    let field_bytes = field.as_bytes();
    let mut results = Vec::new();
    let mut pos = 0;
    while pos + field_bytes.len() + 4 <= data.len() {
        if &data[pos..pos + field_bytes.len()] == field_bytes
            && is_valid_field_match(data, pos, field_bytes.len())
        {
            let val_start = pos + field_bytes.len();
            if let Ok(len_bytes) = data[val_start..val_start + 4].try_into() {
                let val_len = u32::from_be_bytes(len_bytes) as usize;
                if val_len > 0
                    && val_len <= 500
                    && val_start + 4 + val_len <= data.len()
                {
                    let val_bytes = &data[val_start + 4..val_start + 4 + val_len];
                    if val_bytes
                        .iter()
                        .all(|&b| b == 0 || (0x20..=0x7E).contains(&b))
                    {
                        let s = String::from_utf8_lossy(val_bytes)
                            .trim_end_matches('\0')
                            .to_string();
                        if !s.is_empty() {
                            results.push(s);
                        }
                    }
                }
            }
        }
        pos += 1;
    }
    results
}

impl MetadataParser {
    /// 从字节数据解析完整报告
    pub fn parse(data: &[u8], source: &Path) -> Result<StunnerReport, AppError> {
        let mut experiment = ExperimentInfo::default();
        let mut instrument = InstrumentInfo::default();
        let mut layout = LayoutInfo::default();
        let mut user_name = None;
        let mut user_guid = None;

        // 实验信息
        experiment.name = extract_string(data, "experiment_name");
        experiment.id = extract_string(data, "experiment_id");
        experiment.sample_type = extract_string(data, "sample_type");
        experiment.protocol_guid = extract_string(data, "protocol_GUID");

        if let Some(ts) = extract_uint(data, "date") {
            experiment.date = Some(Self::filetime_to_string(ts));
        }
        if let Some(ts) = extract_uint(data, "acquisition_time") {
            experiment.acquisition_time = Some(Self::filetime_to_string(ts));
        }
        if let Some(v) = extract_uint(data, "nr_of_acquisitions") {
            experiment.nr_of_acquisitions = Some(v as u32);
        }
        // processed 是布尔字段（1字节值）
        if let Some(v) = extract_uint(data, "processed") {
            experiment.processed = Some(v != 0);
        }

        // 仪器信息
        instrument.serial_number = extract_string(data, "instrument_S/N");
        instrument.mac_address = extract_string(data, "instrument_MAC");
        instrument.assembly_id = extract_string(data, "assembly_ID");
        instrument.software_type = extract_string(data, "software_type");
        instrument.software_version = extract_string(data, "software_version");
        instrument.control_sw_version = extract_string(data, "control_sw_version");
        instrument.viewer_sw_version = extract_string(data, "viewer_sw_version");
        instrument.firmware = extract_string(data, "firmware");
        instrument.spm = extract_string(data, "SPM");
        if let Some(ts) = extract_uint(data, "calibration date") {
            instrument.calibration_date = Some(Self::filetime_to_string(ts));
        }

        // 布局信息
        if let Some(v) = extract_uint(data, "Layout_size") {
            layout.size = Some(v as u32);
        }
        layout.barcode = extract_string(data, "name_barcode");
        layout.guid = extract_string(data, "layout_GUID");
        layout.disposable_type = extract_string(data, "disposable_type");
        if let Some(v) = extract_uint(data, "disposable_type_id") {
            layout.disposable_type_id = Some(v as u32);
        }
        layout.chip_id_code = extract_string(data, "chip_id_code");
        if let Some(v) = extract_uint(data, "measurement_positions_size") {
            layout.measurement_positions_count = Some(v as u32);
        }

        // 测量位置（可能有多个同名字段）
        let pos_names = extract_all_strings(data, "measurement_position_name");
        for (i, name) in pos_names.into_iter().enumerate() {
            layout.positions.push(MeasurementPosition {
                index: i,
                name,
                x: None,
                y: None,
                source_plate_id: None,
                source_plate_position: None,
            });
        }

        // 用户信息
        user_name = extract_string(data, "name");
        user_guid = extract_string(data, "user_GUID");

        Ok(StunnerReport {
            file_path: source.to_path_buf(),
            version: extract_uint(data, "version").map(|v| v as u32),
            experiment,
            instrument,
            layout,
            absorbance: Default::default(),
            spectra: Vec::new(),
            user_name,
            user_guid,
            extra: HashMap::new(),
        })
    }

    /// Windows FILETIME 转可读字符串
    fn filetime_to_string(filetime: u64) -> String {
        const EPOCH_DIFF: u64 = 116_444_736_000_000_000;
        if filetime <= EPOCH_DIFF {
            return "未知时间".to_string();
        }

        let unix_timestamp = (filetime - EPOCH_DIFF) / 10_000_000;
        let naive_dt = chrono::NaiveDateTime::from_timestamp_opt(unix_timestamp as i64, 0);

        match naive_dt {
            Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => format!("FILETIME: {}", filetime),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filetime_to_string() {
        let result = MetadataParser::filetime_to_string(0);
        assert_eq!(result, "未知时间");
    }

    #[test]
    fn test_parse_empty_metadata() {
        let data = &[];
        let path = Path::new("test.bin");
        let report = MetadataParser::parse(data, path).unwrap();
        assert!(report.experiment.name.is_none());
    }

    #[test]
    fn test_extract_string_from_binary() {
        // 模拟 TLV 字段：key + 4字节长度 + value
        let mut data = Vec::new();
        data.extend_from_slice(b"experiment_name");
        data.extend_from_slice(&5u32.to_be_bytes());
        data.extend_from_slice(b"hello");

        assert_eq!(extract_string(&data, "experiment_name"), Some("hello".to_string()));
        assert_eq!(extract_string(&data, "not_found"), None);
    }

    #[test]
    fn test_extract_uint_from_binary() {
        let mut data = Vec::new();
        data.extend_from_slice(b"version");
        data.extend_from_slice(&4u32.to_be_bytes());
        data.extend_from_slice(&42u32.to_be_bytes());

        assert_eq!(extract_uint(&data, "version"), Some(42));
    }

    #[test]
    fn test_extract_all_strings() {
        let mut data = Vec::new();
        // 第一个位置
        data.extend_from_slice(b"measurement_position_name");
        data.extend_from_slice(&7u32.to_be_bytes());
        data.extend_from_slice(b"blank_A");
        // 第二个位置
        data.extend_from_slice(b"measurement_position_name");
        data.extend_from_slice(&9u32.to_be_bytes());
        data.extend_from_slice(b"sample_B7");

        let names = extract_all_strings(&data, "measurement_position_name");
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "blank_A");
        assert_eq!(names[1], "sample_B7");
    }
}
