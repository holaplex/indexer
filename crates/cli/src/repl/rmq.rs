use docbot::prelude::*;

#[derive(Docbot)]
pub enum RmqCommand {
    /// `todo <todo>`
    /// TODO
    ///
    /// # Arguments
    /// todo: TODO
    Todo(String),
}

pub fn handle(cmd: RmqCommand) -> super::Result {
    todo!()
}
