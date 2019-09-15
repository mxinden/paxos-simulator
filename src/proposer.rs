use crate::{Address, Body, Epoch, Header, Instant, Msg, Value};
use std::collections::VecDeque;

const TIMEOUT: Instant = Instant(5);

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

    pub fn process(&mut self, now: Instant) -> Vec<Msg> {
        let messages: Vec<Msg> = self.inbox.drain(0..).collect();
        let responses: Vec<Msg> = messages
            .into_iter()
            .map(|m| self.process_msg(m, now))
            .flatten()
            .collect();
        if responses.len() != 0 {
            return responses;
        }

        // Go back to /preparing/ on timeout.
        if self
            .state
            .last_progress_at()
            .map(|t| now - t > TIMEOUT)
            .unwrap_or(false)
        {
            self.current_epoch =
                Epoch::new(self.current_epoch.epoch + 1, self.current_epoch.identifier);

            let value = self
                .state
                .value()
                .expect("can't be reached from idle state, thus there is a value");

            self.state = ProposerState::Preparing {
                last_progress_at: now,
                value: value,
                promises: vec![],
            };

            let body = Body::Prepare(self.current_epoch.clone());

            return self.to_all_acceptors(body, now);
        }

        vec![]
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
            ProposerState::Unreachable => unreachable!(),
            ProposerState::Preparing { .. } | ProposerState::Proposing { .. } => {
                self.inbox.push_back(Msg {
                    header,
                    body: Body::Request(value),
                });
                return vec![];
            }
            ProposerState::Idle => {
                self.state = ProposerState::Preparing {
                    last_progress_at: now,
                    value,
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
        let state = std::mem::replace(&mut self.state, ProposerState::Unreachable);
        match state {
            ProposerState::Unreachable => unreachable!(),
            ProposerState::Idle | ProposerState::Proposing { .. } => {
                self.state = state;
                return vec![];
            }
            ProposerState::Preparing {
                value,
                mut promises,
                ..
            } => {
                promises.push(Promise {
                    epoch: promised_epoch,
                    accepted: accepted,
                });

                if promises.len() < self.acceptors.len() / 2 + 1 {
                    self.state = ProposerState::Preparing {
                        last_progress_at: now,
                        value,
                        promises,
                    };
                    return vec![];
                }

                let highest_accepted: Option<(Epoch, Value)> =
                    promises
                        .into_iter()
                        .fold(None, |highest, Promise { accepted, .. }| match highest {
                            Some((h_e, h_v)) => match accepted {
                                Some((a_e, a_v)) => {
                                    if h_e > a_e {
                                        Some((h_e, h_v))
                                    } else {
                                        Some((a_e, a_v))
                                    }
                                }
                                None => Some((h_e, h_v)),
                            },
                            None => accepted,
                        });

                self.state = ProposerState::Proposing {
                    last_progress_at: now,
                    value: highest_accepted.map(|a| a.1).unwrap_or(value),
                    received_accepts: 0,
                };

                let propose_body = Body::Propose(self.current_epoch, self.state.value().unwrap());

                self.to_all_acceptors(propose_body, now)
            }
        }
    }

    fn process_accept(&mut self, epoch: Epoch, now: Instant) -> Vec<Msg> {
        let state = std::mem::replace(&mut self.state, ProposerState::Unreachable);
        match state {
            ProposerState::Unreachable => unimplemented!(),
            ProposerState::Idle | ProposerState::Preparing { .. } => {
                self.state = state;
                return vec![];
            }
            ProposerState::Proposing {
                value,
                received_accepts,
                ..
            } => {
                let received_accepts = received_accepts + 1;

                if received_accepts < self.acceptors.len() / 2 + 1 {
                    self.state = ProposerState::Proposing {
                        value,
                        received_accepts,
                        last_progress_at: now,
                    };
                    return vec![];
                }

                self.current_epoch =
                    Epoch::new(self.current_epoch.epoch + 1, self.current_epoch.identifier);
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
        last_progress_at: Instant,
        value: Value,
        promises: Vec<Promise>,
    },
    Proposing {
        last_progress_at: Instant,
        value: Value,
        received_accepts: usize,
    },
    Unreachable,
}

impl ProposerState {
    fn last_progress_at(&self) -> Option<Instant> {
        match self {
            ProposerState::Unreachable => unreachable!(),
            ProposerState::Idle => None,
            ProposerState::Preparing {
                last_progress_at, ..
            } => Some(*last_progress_at),
            ProposerState::Proposing {
                last_progress_at, ..
            } => Some(*last_progress_at),
        }
    }

    fn value(&self) -> Option<Value> {
        match self {
            ProposerState::Unreachable => unreachable!(),
            ProposerState::Idle => None,
            ProposerState::Preparing { value, .. } => Some(value.clone()),
            ProposerState::Proposing { value, .. } => Some(value.clone()),
        }
    }
}

#[derive(Clone, Debug)]
struct Promise {
    epoch: Epoch,
    accepted: Option<(Epoch, Value)>,
}
