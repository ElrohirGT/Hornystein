use glm::Vec2;

use super::Entity;

#[derive(Debug, PartialEq, Clone)]
pub struct LoliBunny {
    pub position: Vec2,
}

impl Entity<LoliBunny> for LoliBunny {
    fn tick(data: Self) -> (Self, Option<crate::Message>) {
        (data, None)
    }
}
