use paxos_simulator::acceptor::Acceptor;
use paxos_simulator::proposer::Proposer;
use paxos_simulator::{Address, Body, Header, Instant, Msg};
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct Simulator {
    now: Instant,
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
        println!(
            "new simulator with {} proposers and {} acceptors",
            proposers.len(),
            acceptors.len()
        );
        Simulator {
            now: Default::default(),
            proposers,
            acceptors,
            inbox,
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        while (self.inbox.len() != 0 && self.now < Instant(100)) {
            self.tick()
        }

        Ok(())
    }

    fn tick(&mut self) {
        self.now = self.now + 1;
        let now = self.now;
        println!("tick {:?}", now);

        self.dispatch_msgs();

        let mut new_msgs = vec![];
        for (_, p) in self.proposers.iter_mut() {
            new_msgs.append(&mut p.process(now));
        }
        for (_, a) in self.acceptors.iter_mut() {
            new_msgs.append(&mut a.process(now));
        }

        self.inbox.append(&mut new_msgs.into_iter().collect());
    }

    fn dispatch_msgs(&mut self) {
        while let Some(m) = self.inbox.pop_front() {
            println!("dispatching msg '{:?}'", m);
            self.dispatch_msg(m);
        }
    }

    fn dispatch_msg(&mut self, m: Msg) {
        let copy = m.clone();
        match m.body {
            Body::Request(v) => self.proposers.get_mut(&m.header.to).unwrap().receive(copy),
            Body::Prepare(v) => self.acceptors.get_mut(&m.header.to).unwrap().receive(copy),
            Body::Promise(_, _, _) => self.proposers.get_mut(&m.header.to).unwrap().receive(copy),
            Body::Propose(_, _) => unimplemented!(),
            Body::Accept => unimplemented!(),
            Body::Response => unimplemented!(),
        }
    }
}
