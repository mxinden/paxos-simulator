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
            state: ProposerState::WaitingForRequest,
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
        let msg_clone = m.clone();
        println!("processing msg '{:?}'", m);
        match m.body {
            Body::Request(v) => match self.state {
                ProposerState::WaitingForRequest => {
                    self.state = ProposerState::WaitingForSufficientPromises(v, vec![]);

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
                    self.inbox.push_back(msg_clone);
                    return vec![];
                }
            },
            Body::Promise(promised_epoch, accepted_epoch, accepted_value) => {
                match std::mem::replace(&mut self.state, ProposerState::WaitingForRequest) {
                    ProposerState::WaitingForSufficientPromises(value, mut received_promises) => {
                        received_promises.push((promised_epoch, accepted_epoch, accepted_value));

                        if received_promises.len() < self.acceptors.len() / 2 + 1 {
                            println!("not a majority yet");
                            self.state = ProposerState::WaitingForSufficientPromises(
                                value,
                                received_promises,
                            );
                            return vec![];
                        }

                        // Let's determine if we got promises for our proposal, or
                        // if there was something higher.
                        let (highest_epoch, highest_epoch_value) =
                            received_promises.into_iter().fold(
                                (self.current_epoch, value),
                                |(highest_epoch, v), promise| -> (Epoch, Value) {
                                    if promise.1.map(|e| e > highest_epoch).unwrap_or(false) {
                                        return (promise.1.unwrap(), promise.2.unwrap());
                                    }
                                    (highest_epoch, v)
                                },
                            );

                        self.state = ProposerState::WaitingForSufficientAccepts(
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
                        println!("got a promise even though we aren't waiting for any");
                        return vec![];
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
}

enum ProposerState {
    WaitingForRequest,
    /// Value to propose and promises received so far.
    WaitingForSufficientPromises(Value, Vec<(Epoch, Option<Epoch>, Option<Value>)>),
    WaitingForSufficientAccepts(Epoch, Value, u32),
}
