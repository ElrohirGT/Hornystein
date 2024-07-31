use crate::Message;

mod lolibunny;
pub use lolibunny::*;

trait Entity<State> {
    fn tick(data: State) -> (State, Option<Message>);
}
