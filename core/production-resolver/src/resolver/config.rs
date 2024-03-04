pub struct LineConfig {
    id: i64,
    selected_tools: [db_api::Tool; 2],
    available_tools: [[db_api::Tool; 3]; 2],
    tool_change_time: i32,
}

impl Default for LineConfig {
    fn default() -> Self {
        use db_api::Tool as Tl;
        Self {
            id: 1,
            selected_tools: [Tl::T1, Tl::T1],
            available_tools: [
                [Tl::T1, Tl::T2, Tl::T3],
                [Tl::T1, Tl::T2, Tl::T3],
            ],
            tool_change_time: 30,
        }
    }
}
