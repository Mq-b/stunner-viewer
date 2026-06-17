/// 单条光谱数据
#[derive(Debug, Clone)]
pub struct Spectrum {
    /// 关联的测量位置索引
    pub position_index: usize,
    /// 通道标签
    pub channel_label: Option<String>,
    /// 波长值（nm）
    pub wavelengths: Vec<f32>,
    /// 吸光度/强度值
    pub values: Vec<f32>,
}

impl Spectrum {
    /// 波长点数（Stunner 固定 601）
    pub const POINT_COUNT: usize = 601;
    /// 起始波长（nm）
    pub const WL_START: f32 = 190.0;
    /// 结束波长（nm）
    pub const WL_END: f32 = 790.0;

    /// 创建新的光谱数据
    ///
    /// 波长轴自动生成，点数与 values 长度一致。
    pub fn new(position_index: usize, values: Vec<f32>) -> Self {
        let count = values.len();
        let wavelengths = if count > 0 {
            let step = (Self::WL_END - Self::WL_START) / (count as f32 - 1.0).max(1.0);
            (0..count).map(|i| Self::WL_START + i as f32 * step).collect()
        } else {
            Self::default_wavelengths()
        };
        Self {
            position_index,
            channel_label: None,
            wavelengths,
            values,
        }
    }

    /// 生成默认波长轴（190-790nm，601点）
    pub fn default_wavelengths() -> Vec<f32> {
        (0..Self::POINT_COUNT)
            .map(|i| Self::WL_START + i as f32)
            .collect()
    }

    /// 获取数据点数量
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// 获取最大值
    pub fn max_value(&self) -> f32 {
        self.values
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max)
    }

    /// 获取最小值
    pub fn min_value(&self) -> f32 {
        self.values
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min)
    }
}
