use crate::{Address, Epoch, Header, Instant, Msg, Value, Node};
use std::collections::VecDeque;
use super::Body;

#[derive(Default, Debug)]
pub struct Acceptor {
    address: Address,
    promised_epoch: Option<Epoch>,
    accepted: Option<(Epoch, Value)>,
    inbox: VecDeque<Msg<Body>>,
}

impl Node<Body> for Acceptor {
    fn receive(&mut self, m: Msg<Body>) {
        self.inbox.push_back(m);
    }

    fn process(&mut self, now: Instant) -> Vec<Msg<Body>> {
        self.process(now)
    }
}

impl crate::Acceptor<Body> for Acceptor {}

impl Acceptor {
    pub fn new(address: Address) -> Self {
        Acceptor{
            address,
            promised_epoch: None,
            accepted: None,
            inbox: VecDeque::new(),
        }
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }

    pub fn process(&mut self, now: Instant) -> Vec<Msg<Body>> {
        let messages: Vec<Msg<Body>> = self.inbox.drain(0..).collect();
        messages
            .into_iter()
            .map(|m| self.process_msg(m, now))
            .flatten()
            .collect()
    }

    fn process_msg(&mut self, m: Msg<Body>, now: Instant) -> Vec<Msg<Body>> {
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
                        at: now + 1,
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
                        at: now + 1,
                    },
                    body: Body::Accept(proposed_epoch),
                }];
            }
            _ => unimplemented!(),
        }
    }
}
