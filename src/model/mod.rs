pub mod experiment;
pub mod instrument;
pub mod measurement;
pub mod spectrum;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::parser::tlv::TlvValue;

pub use experiment::ExperimentInfo;
pub use instrument::InstrumentInfo;
pub use measurement::{AbsorbanceData, LayoutInfo, MeasurementPosition};
pub use spectrum::Spectrum;

/// 单个 bin 文件的完整解析结果
#[derive(Debug, Clone)]
pub struct StunnerReport {
    /// 文件路径
    pub file_path: PathBuf,
    /// 文件格式版本
    pub version: Option<u32>,
    /// 实验信息
    pub experiment: ExperimentInfo,
    /// 仪器信息
    pub instrument: InstrumentInfo,
    /// 布局信息
    pub layout: LayoutInfo,
    /// 吸光度数据
    pub absorbance: AbsorbanceData,
    /// 所有光谱数据
    pub spectra: Vec<Spectrum>,
    /// 用户信息
    pub user_name: Option<String>,
    pub user_guid: Option<String>,
    /// 无法归类的原始键值对
    pub extra: HashMap<String, TlvValue>,
}

impl StunnerReport {
    /// 获取文件名
    pub fn filename(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知文件")
    }
}
