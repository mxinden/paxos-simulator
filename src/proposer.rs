use crate::{Address, Msg};

pub struct Proposer {
    acceptors: Vec<Address>,
    decided_value: Option<String>,
}

impl Proposer {
    pub fn new(acceptors: Vec<Address>) -> Self {
        Self {
            acceptors,
            decided_value: None,
        }
    }

    pub fn process(m: Msg) -> Vec<Msg> {
        match m {
            _ => unimplemented!(),
        }
    }

    pub fn decided_value(&self) -> Option<String> {
        self.decided_value.clone()
    }
}
