use crate::{Address, Body, Epoch, Header, Instant, Msg};
use std::collections::VecDeque;

pub struct Proposer {
    address: Address,
    acceptors: Vec<Address>,
    inbox: VecDeque<Msg>,
    current_epoch: Epoch,
}

impl Proposer {
    pub fn new(address: Address, acceptors: Vec<Address>) -> Self {
        Self {
            address,
            acceptors,
            inbox: Default::default(),
            // TODO: This way all proposers start with the same epoch. Is that a
            // good idea?
            current_epoch: Epoch::default(),
        }
    }

    /// Receive adds the given message to the incoming-messages buffer. It is
    /// *not* allowed to do any kind of processing.
    pub fn receive(&mut self, m: Msg) {
        self.inbox.push_back(m);
    }

    pub fn process(&mut self, now: Instant) -> Vec<Msg> {
        let messages: Vec<Msg>= self.inbox
            .drain(0..).collect();
        messages.into_iter()
            .map(|m| self.process_msg(m))
            .flatten()
            .collect()
    }

    fn process_msg(&mut self, m: Msg) -> Vec<Msg> {
        match m.body {
            Body::Request(v) => {
                let body = Body::Prepare(self.current_epoch.clone());

                return self
                    .acceptors
                    .iter()
                    .map(|a| Msg {
                        header: Header {
                            from: self.address.clone(),
                            to: a.clone(),
                        },
                        body: body.clone(),
                    })
                    .collect();
            }
            _ => unimplemented!(),
        }
    }
}
