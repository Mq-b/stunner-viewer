/// 测量位置
#[derive(Debug, Clone)]
pub struct MeasurementPosition {
    /// 位置索引
    pub index: usize,
    /// 位置名称
    pub name: String,
    /// X 坐标
    pub x: Option<f32>,
    /// Y 坐标
    pub y: Option<f32>,
    /// 来源板 ID
    pub source_plate_id: Option<String>,
    /// 来源板位置
    pub source_plate_position: Option<u32>,
}

/// 吸光度数据
#[derive(Debug, Clone, Default)]
pub struct AbsorbanceData {
    /// 吸光度阈值数组
    pub thresholds: Vec<f32>,
    /// 吸光度过渡数组
    pub transitions: Vec<f32>,
    /// 标称光程长度1
    pub nominal_path_length1: Option<f32>,
    /// 标称光程长度2
    pub nominal_path_length2: Option<f32>,
    /// OD 限制1
    pub od_limit1: Option<f32>,
    /// OD 限制2
    pub od_limit2: Option<f32>,
}

/// 布局信息
#[derive(Debug, Clone, Default)]
pub struct LayoutInfo {
    /// 布局大小
    pub size: Option<u32>,
    /// 条码
    pub barcode: Option<String>,
    /// GUID
    pub guid: Option<String>,
    /// 一次性耗材类型
    pub disposable_type: Option<String>,
    /// 一次性耗材类型 ID
    pub disposable_type_id: Option<u32>,
    /// 芯片 ID 代码
    pub chip_id_code: Option<String>,
    /// 测量位置数量
    pub measurement_positions_count: Option<u32>,
    /// 测量位置列表
    pub positions: Vec<MeasurementPosition>,
}
