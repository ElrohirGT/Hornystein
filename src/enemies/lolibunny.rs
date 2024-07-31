use super::Entity;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LoliBunny;

impl Entity<LoliBunny> for LoliBunny {
    fn tick(data: Self) -> (Self, Option<crate::Message>) {
        (Self, None)
    }
}
