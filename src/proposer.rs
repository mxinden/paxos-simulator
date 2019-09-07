use crate::{Address, Body, Epoch, Header, Instant, Msg, Value};
use std::collections::VecDeque;

/// A sequential proposer, handling a single request at a time.
pub struct Proposer {
    address: Address,
    acceptors: Vec<Address>,
    inbox: VecDeque<Msg>,
    current_epoch: Epoch,
    max_epoch_received: Option<Epoch>,
    state: ProposerState,
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
            max_epoch_received: None,
            state: ProposerState::Idle,
        }
    }

    /// Receive adds the given message to the incoming-messages buffer. It is
    /// *not* allowed to do any kind of processing.
    pub fn receive(&mut self, m: Msg) {
        self.inbox.push_back(m);
    }

    // TODO: Implement timeout mechanism.
    pub fn process(&mut self, now: Instant) -> Vec<Msg> {
        let messages: Vec<Msg> = self.inbox.drain(0..).collect();
        messages
            .into_iter()
            .map(|m| self.process_msg(m))
            .flatten()
            .collect()
    }

    fn process_msg(&mut self, m: Msg) -> Vec<Msg> {
        println!("processing msg '{:?}'", m);
        match m.body {
            Body::Request(v) => self.process_request(m.header, v),
            Body::Promise(pe, ae, av) => self.process_promise(pe, ae, av),
            Body::Accept(epoch) => self.process_accept(epoch),
            Body::Prepare(_) | Body::Propose(_, _) | Body::Response(_) => unimplemented!(),
        }
    }

    fn process_request(&mut self, header: Header, value: Value) -> Vec<Msg> {
        match self.state {
            ProposerState::Idle => {
                self.state = ProposerState::Preparing(value, vec![]);

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
            _ => {
                println!("already got a request in flight, delaying new one");
                self.inbox.push_back(Msg {
                    header,
                    body: Body::Request(value),
                });
                return vec![];
            }
        }
    }

    fn process_promise(
        &mut self,
        promised_epoch: Epoch,
        accepted_epoch: Option<Epoch>,
        accepted_value: Option<Value>,
    ) -> Vec<Msg> {
        let state = std::mem::replace(&mut self.state, ProposerState::Idle);
        match state {
            ProposerState::Preparing(value, mut received_promises) => {
                received_promises.push((promised_epoch, accepted_epoch, accepted_value));

                if received_promises.len() < self.acceptors.len() / 2 + 1 {
                    println!("not a majority yet");
                    self.state =
                        ProposerState::Preparing(value, received_promises);
                    return vec![];
                }

                // Let's determine if we got promises for our proposal, or
                // if there was something higher.
                let (highest_epoch, highest_epoch_value) = received_promises.into_iter().fold(
                    (self.current_epoch, value),
                    |(highest_epoch, v), promise| -> (Epoch, Value) {
                        if promise.1.map(|e| e > highest_epoch).unwrap_or(false) {
                            return (promise.1.unwrap(), promise.2.unwrap());
                        }
                        (highest_epoch, v)
                    },
                );

                self.state = ProposerState::Proposing(
                    highest_epoch,
                    highest_epoch_value.clone(),
                    0,
                );

                let propose_body = Body::Propose(highest_epoch, highest_epoch_value);

                return self
                    .acceptors
                    .iter()
                    .map(|a| Msg {
                        header: Header {
                            from: self.address.clone(),
                            to: a.clone(),
                        },
                        body: propose_body.clone(),
                    })
                    .collect();
            }
            _ => {
                self.state = state;
                println!("got a promise even though we aren't waiting for any");
                return vec![];
            }
        }
    }

    fn process_accept(&mut self, epoch: Epoch) -> Vec<Msg> {
        let state = std::mem::replace(&mut self.state, ProposerState::Idle);
        match state {
            ProposerState::Proposing(expected_epoch, value, count) => {
                if epoch != expected_epoch {
                    panic!("got unexpected epoch");
                }

                let count = count + 1;

                if count < self.acceptors.len() / 2 + 1 {
                    self.state =
                        ProposerState::Proposing(expected_epoch, value, count);
                    return vec![];
                }

                // TODO: Should we be increasing the epoch?
                self.state = ProposerState::Idle;

                return vec![Msg {
                    header: Header {
                        from: self.address.clone(),
                        // TODO: We need to track the client address along the way.
                        to: Address::new(""),
                    },
                    body: Body::Response(value),
                }];
            }
            _ => {
                println!("got an accept even though we aren't waiting for any");
                return vec![];
            }
        }
    }
}

#[derive(Clone, Debug)]
enum ProposerState {
    Idle,
    /// Value to propose and promises received so far.
    Preparing(Value, Vec<(Epoch, Option<Epoch>, Option<Value>)>),
    Proposing(Epoch, Value, usize),
}
