use crate::{Address, Body, Epoch, Header, Instant, Msg, Value};
use std::collections::VecDeque;

/// A sequential proposer, handling a single request at a time.
#[derive(Debug)]
pub struct Proposer {
    address: Address,
    pub acceptors: Vec<Address>,
    inbox: VecDeque<Msg>,
    current_epoch: Epoch,
    state: ProposerState,
}

impl Proposer {
    pub fn new(address: Address, initial_epoch: Epoch, acceptors: Vec<Address>) -> Self {
        Self {
            address,
            acceptors,
            inbox: Default::default(),
            // TODO: This way all proposers start with the same epoch. Is that a
            // good idea?
            current_epoch: initial_epoch,
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
            .map(|m| self.process_msg(m, now))
            .flatten()
            .collect()
    }

    fn process_msg(&mut self, m: Msg, now: Instant) -> Vec<Msg> {
        match m.body {
            Body::Request(v) => self.process_request(m.header, v, now),
            Body::Promise(promised_epoch, accepted) => {
                self.process_promise(promised_epoch, accepted, now)
            }
            Body::Accept(epoch) => self.process_accept(epoch, now),
            Body::Prepare(_) | Body::Propose(_, _) | Body::Response(_) => unimplemented!(),
        }
    }

    fn process_request(&mut self, header: Header, value: Value, now: Instant) -> Vec<Msg> {
        match self.state {
            ProposerState::Preparing { .. } | ProposerState::Proposing(_, _, _) => {
                println!("already got a request in flight, delaying new one");
                self.inbox.push_back(Msg {
                    header,
                    body: Body::Request(value),
                });
                return vec![];
            }
            ProposerState::Idle => {
                self.state = ProposerState::Preparing {
                    initial_e: self.current_epoch.clone(),
                    initial_v: value,
                    promises: vec![],
                };

                let body = Body::Prepare(self.current_epoch.clone());

                self.to_all_acceptors(body, now)
            }
        }
    }

    fn process_promise(
        &mut self,
        promised_epoch: Epoch,
        accepted: Option<(Epoch, Value)>,
        now: Instant,
    ) -> Vec<Msg> {
        let state = std::mem::replace(&mut self.state, ProposerState::Idle);
        match state {
            ProposerState::Idle | ProposerState::Proposing(_, _, _) => {
                self.state = state;
                println!("got a promise even though we aren't waiting for any");
                return vec![];
            }
            ProposerState::Preparing {
                initial_e,
                initial_v,
                mut promises,
            } => {
                promises.push(Promise {
                    epoch: promised_epoch,
                    accepted: accepted,
                });

                if promises.len() < self.acceptors.len() / 2 + 1 {
                    println!("not a majority yet");
                    self.state = ProposerState::Preparing {
                        initial_e,
                        initial_v,
                        promises,
                    };
                    return vec![];
                }

                // Let's determine if we got promises for our proposal, or
                // if there was something higher.
                let (epoch, value) = promises.into_iter().fold(
                    (initial_e, initial_v),
                    |(accu_e, accu_v), Promise { accepted, .. }| -> (Epoch, Value) {
                        if accepted.clone().map(|(e, _)| e > accu_e).unwrap_or(false) {
                            return accepted.unwrap();
                        }
                        (accu_e, accu_v)
                    },
                );

                self.state = ProposerState::Proposing(epoch, value.clone(), 0);

                let propose_body = Body::Propose(epoch, value);

                self.to_all_acceptors(propose_body, now)
            }
        }
    }

    fn process_accept(&mut self, epoch: Epoch, now: Instant) -> Vec<Msg> {
        let state = std::mem::replace(&mut self.state, ProposerState::Idle);
        match state {
            ProposerState::Proposing(expected_epoch, value, count) => {
                if epoch != expected_epoch {
                    panic!("got unexpected epoch");
                }

                let count = count + 1;

                if count < self.acceptors.len() / 2 + 1 {
                    self.state = ProposerState::Proposing(expected_epoch, value, count);
                    return vec![];
                }

                // TODO: Should we be increasing the epoch?
                self.state = ProposerState::Idle;

                return vec![Msg {
                    header: Header {
                        from: self.address.clone(),
                        // TODO: We need to track the client address along the way.
                        to: Address::new(""),
                        at: now + 1,
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

    fn to_all_acceptors(&mut self, b: Body, now: Instant) -> Vec<Msg> {
        self.acceptors
            .iter()
            .map(|a| Msg {
                header: Header {
                    from: self.address.clone(),
                    to: a.clone(),
                    at: now + 1,
                },
                body: b.clone(),
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
enum ProposerState {
    Idle,
    /// Epoch and value to propose and promises received so far.
    Preparing {
        initial_e: Epoch,
        initial_v: Value,
        promises: Vec<Promise>,
    },
    Proposing(Epoch, Value, usize),
}

#[derive(Clone, Debug)]
struct Promise {
    epoch: Epoch,
    accepted: Option<(Epoch, Value)>,
}
