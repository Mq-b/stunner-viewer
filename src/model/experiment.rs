/// 实验信息
#[derive(Debug, Clone, Default)]
pub struct ExperimentInfo {
    /// 实验名称
    pub name: Option<String>,
    /// 实验 ID
    pub id: Option<String>,
    /// 实验日期
    pub date: Option<String>,
    /// 样品类型
    pub sample_type: Option<String>,
    /// 是否已处理
    pub processed: Option<bool>,
    /// 采集时间
    pub acquisition_time: Option<String>,
    /// 采集次数
    pub nr_of_acquisitions: Option<u32>,
    /// 协议 GUID
    pub protocol_guid: Option<String>,
}
