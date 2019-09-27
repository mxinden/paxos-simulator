use crate::{Address, Body, Epoch, Header, Instant, Msg, Node, Value};
use std::collections::VecDeque;

const TIMEOUT: Instant = Instant(10);

/// A sequential proposer, handling a single request at a time.
#[derive(Debug)]
pub struct Proposer {
    address: Address,
    pub acceptors: Vec<Address>,
    inbox: VecDeque<Msg>,
    epoch: Epoch,
    state: ProposerState,
}

impl Node for Proposer {
    fn receive(&mut self, m: Msg) {
        self.inbox.push_back(m);
    }

    fn process(&mut self, now: Instant) -> Vec<Msg> {
        self.process(now)
    }
}

impl crate::Proposer for Proposer{}

impl Proposer {
    pub fn new(address: Address, initial_epoch: Epoch, acceptors: Vec<Address>) -> Self {
        Self {
            address,
            acceptors,
            inbox: Default::default(),
            epoch: initial_epoch,
            state: ProposerState::Idle,
        }
    }

    fn process(&mut self, now: Instant) -> Vec<Msg>{
        let messages: Vec<Msg> = self.inbox.drain(0..).collect();
        let responses: Vec<Msg> = messages
            .into_iter()
            .map(|m| self.process_msg(m, now))
            .flatten()
            .collect();
        if !responses.is_empty() {
            // We made progress, thus returning.
            return responses;
        }

        // Check whether we are still within the timeout.
        if self
            .state
            .last_progress_at()
            .map(|t| now - t < TIMEOUT)
            .unwrap_or(true)
        {
            return vec![];
        }

        // We timed out - going back to preparing.

        self.epoch = Epoch::new(self.epoch.epoch + 1, self.epoch.identifier);

        let value = self
            .state
            .value()
            .expect("can't be reached from idle state, thus there is a value");

        self.state = ProposerState::Preparing {
            last_progress_at: now,
            value,
            promises: vec![],
        };

        let body = Body::Prepare(self.epoch);

        return self.broadcast_to_acceptors(body, now);
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

                let body = Body::Prepare(self.epoch);

                self.broadcast_to_acceptors(body, now)
            }
        }
    }

    fn process_promise(
        &mut self,
        promised_epoch: Epoch,
        accepted: Option<(Epoch, Value)>,
        now: Instant,
    ) -> Vec<Msg> {
        // Ignore any messages outside our current epoch.
        if promised_epoch != self.epoch {
            return vec![];
        }

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
                    accepted,
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

                let propose_body = Body::Propose(self.epoch, self.state.value().unwrap());

                self.broadcast_to_acceptors(propose_body, now)
            }
        }
    }

    fn process_accept(&mut self, epoch: Epoch, now: Instant) -> Vec<Msg> {
        // Ignore any messages outside our current epoch.
        if epoch != self.epoch {
            return vec![];
        }

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

                self.epoch = Epoch::new(self.epoch.epoch + 1, self.epoch.identifier);
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

    fn broadcast_to_acceptors(&mut self, b: Body, now: Instant) -> Vec<Msg> {
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
