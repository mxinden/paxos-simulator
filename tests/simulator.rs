use paxos_simulator::acceptor::Acceptor;
use paxos_simulator::proposer::Proposer;
use paxos_simulator::{Address, Body, Instant, Msg};
use std::collections::{HashMap, VecDeque};

#[derive(Default, Debug)]
pub struct Simulator {
    now: Instant,
    proposers: HashMap<Address, Proposer>,
    acceptors: HashMap<Address, Acceptor>,
    inbox: VecDeque<Msg>,
    pub responses: Vec<Msg>,
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
        let s = Simulator {
            now: Default::default(),
            proposers,
            acceptors,
            inbox,
            responses: vec![],
        };

        println!("{:?}", s);

        return s;
    }

    pub fn run(&mut self) -> Result<(), ()> {
        while self.inbox.len() != 0 && self.now < Instant(100) {
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
        match m.body {
            Body::Request(_) => self.dispatch_msg_to_proposer(m),
            Body::Prepare(_) => self.dispatch_msg_to_acceptor(m),
            Body::Promise(_, _) => self.dispatch_msg_to_proposer(m),
            Body::Propose(_, _) => self.dispatch_msg_to_acceptor(m),
            Body::Accept(_) => self.dispatch_msg_to_proposer(m),
            Body::Response(_) => self.responses.push(m),
        };
    }

    fn dispatch_msg_to_proposer(&mut self, m: Msg) {
        println!("dispatching {:?}", m);
        self.proposers.get_mut(&m.header.to).unwrap().receive(m);
    }

    fn dispatch_msg_to_acceptor(&mut self, m: Msg) {
        self.acceptors.get_mut(&m.header.to).unwrap().receive(m);
    }
}
