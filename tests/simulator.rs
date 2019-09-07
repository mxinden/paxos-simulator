use paxos_simulator::acceptor::Acceptor;
use paxos_simulator::proposer::Proposer;
use paxos_simulator::{Address, Body, Header, Msg};
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct Simulator {
    now: u64,
    pub proposers: HashMap<Address, Proposer>,
    acceptors: HashMap<Address, Acceptor>,
    inbox: VecDeque<Msg>,
}

impl Simulator {
    pub fn new(
        proposers: HashMap<Address, Proposer>,
        acceptors: HashMap<Address, Acceptor>,
        inbox: VecDeque<Msg>,
    ) -> Simulator {
        Simulator {
            now: Default::default(),
            proposers,
            acceptors,
            inbox,
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        while let Some(m) = self.inbox.pop_front() {
            self.dispatch_msg(m);
        }
        Ok(())
    }

    pub fn dispatch_msg(&mut self, m: Msg) {
        let copy = m.clone();
        match m.body {
            Body::Request(v) => self
                .proposers
                .get_mut(&m.header.to)
                .unwrap()
                .receive(copy),
            Body::Prepare(_) => {
                for (_, a) in self.acceptors.iter_mut() {
                    self.inbox.append(&mut a.process(copy.clone()).into())
                }
            }
            Body::Promise(_, _, _) => unimplemented!(),
            Body::Propose => unimplemented!(),
            Body::Accept => unimplemented!(),
            Body::Response => unimplemented!(),
        }
    }
}
