use crate::{Address, Body, Header, Instant, Msg, Value};

#[derive(Default)]
pub struct Acceptor {
    address: Address,
    promised_epoch: Option<Instant>,
    accepted_epoch: Option<Instant>,
    accepted_value: Option<Value>,
}

impl Acceptor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, m: Msg) -> Vec<Msg> {
        match m.body {
            Body::Prepare(i) => {
                if self.promised_epoch.map(|e| e > i).unwrap_or(false) {
                    return vec![];
                }

                // TODO: Set own promised epoch.

                return vec![Msg {
                    header: Header {
                        from: self.address.clone(),
                        to: m.header.from,
                    },
                    body: Body::Promise(
                        Some(i),
                        self.accepted_epoch.clone(),
                        self.accepted_value.clone(),
                    ),
                }];
            }
            _ => unimplemented!(),
        }
    }
}
