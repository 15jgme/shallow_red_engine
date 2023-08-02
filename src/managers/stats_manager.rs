#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct Statistics {
    pub all_nodes: i32,
    pub searched_nodes: i32,
    pub caches_used: i32,
    pub time_ms: f32,
    pub depth_reached: u8,
}

pub struct StatisticsInputGrouping{
    
}