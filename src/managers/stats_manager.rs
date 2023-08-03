use std::ops::AddAssign;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub all_nodes: i32,
    pub searched_nodes: i32,
    pub caches_used: i32,
}

impl  AddAssign for Statistics {
    fn add_assign(&mut self, rhs: Self) {
        self.all_nodes += rhs.all_nodes;
        self.searched_nodes += rhs.searched_nodes;
        self.caches_used += rhs.caches_used;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct StatisticsDepth {
    pub depth_reached: u8,
    pub time_ms: f32,
}

#[cfg(test)]
mod tests{
    use super::Statistics;

    #[test]
    fn test_stats_add_assign(){
        let mut a: Statistics = Statistics { all_nodes: 1, searched_nodes: 2, caches_used: 3 };
        let b: Statistics = Statistics { all_nodes: 4, searched_nodes: 5, caches_used: 6 };

        a += b;
        assert_eq!(a, Statistics{ all_nodes: 5, searched_nodes: 7, caches_used: 9 })
    }
}