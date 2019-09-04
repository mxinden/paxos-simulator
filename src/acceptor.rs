use crate::Msg;

pub struct Acceptor {}

impl Acceptor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process(m: Msg) -> Vec<Msg> {
        vec![]
    }
}
