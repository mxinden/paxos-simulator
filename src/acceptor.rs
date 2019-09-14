use crate::{Address, Body, Epoch, Header, Instant, Msg, Value};
use std::collections::VecDeque;

#[derive(Default)]
pub struct Acceptor {
    address: Address,
    promised_epoch: Option<Epoch>,
    accepted: Option<(Epoch, Value)>,
    inbox: VecDeque<Msg>,
}

impl Acceptor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Receive adds the given message to the incoming-messages buffer. It is
    /// *not* allowed to do any kind of processing.
    pub fn receive(&mut self, m: Msg) {
        self.inbox.push_back(m);
    }

    pub fn process(&mut self, _now: Instant) -> Vec<Msg> {
        let messages: Vec<Msg> = self.inbox.drain(0..).collect();
        messages
            .into_iter()
            .map(|m| self.process_msg(m))
            .flatten()
            .collect()
    }

    fn process_msg(&mut self, m: Msg) -> Vec<Msg> {
        match m.body {
            Body::Prepare(i) => {
                if self.promised_epoch.map(|e| e > i).unwrap_or(false) {
                    return vec![];
                }

                self.promised_epoch = Some(i);

                return vec![Msg {
                    header: Header {
                        from: self.address.clone(),
                        to: m.header.from,
                    },
                    body: Body::Promise(i, self.accepted.clone()),
                }];
            }
            Body::Propose(proposed_epoch, value) => {
                if self
                    .promised_epoch
                    .map(|e| e > proposed_epoch)
                    .unwrap_or(false)
                {
                    return vec![];
                }

                self.accepted = Some((proposed_epoch, value));

                return vec![Msg {
                    header: Header {
                        from: self.address.clone(),
                        to: m.header.from,
                    },
                    body: Body::Accept(proposed_epoch),
                }];
            }
            _ => unimplemented!(),
        }
    }
}
