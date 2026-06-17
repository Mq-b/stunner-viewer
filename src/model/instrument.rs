/// 仪器信息
#[derive(Debug, Clone, Default)]
pub struct InstrumentInfo {
    /// 序列号
    pub serial_number: Option<String>,
    /// MAC 地址
    pub mac_address: Option<String>,
    /// 装配 ID
    pub assembly_id: Option<String>,
    /// 软件类型
    pub software_type: Option<String>,
    /// 软件版本
    pub software_version: Option<String>,
    /// 控制软件版本
    pub control_sw_version: Option<String>,
    /// 查看器软件版本
    pub viewer_sw_version: Option<String>,
    /// 固件版本
    pub firmware: Option<String>,
    /// SPM
    pub spm: Option<String>,
    /// 校准日期
    pub calibration_date: Option<String>,
}
