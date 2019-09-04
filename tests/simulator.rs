use paxos_simulator::acceptor::Acceptor;
use paxos_simulator::proposer::Proposer;
use paxos_simulator::{Address, Msg};
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct Simulator {
    now: u64,
    proposers: HashMap<Address, Proposer>,
    acceptors: HashMap<Address, Acceptor>,
    inbox: std::collections::VecDeque<Msg>,
}

impl Simulator {
    pub fn new() -> Simulator {
        Simulator::default()
    }

    pub fn run(&mut self) -> Result<(), ()> {
        while let Some(m) = self.inbox.pop_front() {};
        Ok(())
    }

    pub fn dispatch_msg(&mut self, m: Msg) {
        match m {
            _ => unimplemented!(),
        }
    }
}
