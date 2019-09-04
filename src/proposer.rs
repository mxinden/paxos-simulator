use crate::{Address, Body, Header, Msg, Instant};

pub struct Proposer {
    address: Address,
    acceptors: Vec<Address>,
    decided_value: Option<String>,
    now: Instant,
}

impl Proposer {
    pub fn new(address: Address, acceptors: Vec<Address>) -> Self {
        Self {
            address,
            acceptors,
            decided_value: None,
            now: Default::default(),
        }
    }

    pub fn process(&mut self, m: Msg) -> Vec<Msg> {
        match m.body {
            Body::Request(v) => {
                let body = Body::Prepare(self.now.clone());

                return self.acceptors.iter().map(|a| Msg {
                    header: Header {
                        from: self.address.clone(),
                        to: a.clone(),
                    },
                    body: body.clone(),
                }).collect();
            }
            _ => unimplemented!(),
        }
    }

    pub fn decided_value(&self) -> Option<String> {
        self.decided_value.clone()
    }
}
