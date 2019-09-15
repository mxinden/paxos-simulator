use paxos_simulator::acceptor::Acceptor;
use paxos_simulator::proposer::Proposer;
use paxos_simulator::{Address, Body, Instant, Msg, Value};
use std::collections::HashMap;

// Needs to be larger than proposer.rs/TIMEOUT.
const TIMEOUT: Instant = Instant(10);

#[derive(Default, Debug)]
pub struct Simulator {
    now: Instant,
    proposers: HashMap<Address, Proposer>,
    acceptors: HashMap<Address, Acceptor>,
    inbox: Vec<Msg>,
    responses: Vec<Msg>,
    amount_requests: usize,
    // The simulator needs to be able to determine when the simulation is done,
    // thus not making any more progress. One could terminate once no messages
    // are being transferred anymore. But this would break proposer timeouts.
    // Instead let's wait a bit longer once there are no more messages.
    // `last_progress_at` is there to track the above.
    last_progress_at: Instant,
    /// Log lines collected to be printed on failure.
    pub log: Vec<String>,
}

impl Simulator {
    pub fn new(
        proposers: HashMap<Address, Proposer>,
        acceptors: HashMap<Address, Acceptor>,
        inbox: Vec<Msg>,
    ) -> Simulator {
        let amount_requests = inbox.len();
        Simulator {
            now: Default::default(),
            last_progress_at: Default::default(),
            proposers,
            acceptors,
            inbox,
            responses: vec![],
            amount_requests,
            log: Default::default(),
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
        self.log.push(format!(
            "=== New simulation | proposers: {} | acceptors: {} | initial inbox: {}",
            self.proposers.len(),
            self.acceptors.len(),
            self.inbox.len()
        ));

        println!("inbox: {:?}", self.inbox);

        loop {
            self.tick();

            // Check if there is any progress.
            if self.inbox.len() == 0 && self.now - self.last_progress_at > TIMEOUT {
                break;
            }

            // Safety measure to prevent infinite loops.
            if self.now > Instant(100000) {
                break;
            }
        }

        Ok(())
    }

    fn tick(&mut self) {
        self.now = self.now + 1;
        self.log.push(format!("tick {:?}", self.now));

        // Dispatch messages.
        self.inbox.sort_unstable();
        self.dispatch_msgs();

        // Have entities process messages.
        let mut new_msgs = vec![];
        for (_, p) in self.proposers.iter_mut() {
            new_msgs.append(&mut p.process(self.now));
        }
        for (_, a) in self.acceptors.iter_mut() {
            new_msgs.append(&mut a.process(self.now));
        }

        // Producing new messages is equal to overall progress.
        if new_msgs.len() != 0 {
            self.last_progress_at = self.now;
        }

        self.inbox.append(&mut new_msgs.into_iter().collect());
    }

    fn dispatch_msgs(&mut self) {
        while self
            .inbox
            .get(0)
            .map(|m| m.header.at <= self.now)
            .unwrap_or(false)
        {
            let m = self.inbox.remove(0);
            self.log.push(format!("dispatching msg '{:?}'", m));
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
        self.proposers.get_mut(&m.header.to).unwrap().receive(m);
    }

    fn dispatch_msg_to_acceptor(&mut self, m: Msg) {
        self.acceptors.get_mut(&m.header.to).unwrap().receive(m);
    }

    pub fn ensure_correctness(&self) -> Result<(), String> {
        println!("{:?}", self.responses);
        if self.responses.len() != self.amount_requests {
            return Err(format!(
                "expected {} responses, got {} responses",
                self.amount_requests,
                self.responses.len(),
            ));
        }

        let final_values = self
            .responses
            .iter()
            .map(|r| match &r.body {
                Body::Response(v) => v.clone(),
                _ => unreachable!(),
            })
            .collect::<Vec<Value>>();

        println!("{:?}", final_values);

        let mut unique_final_values = final_values.clone();
        unique_final_values.sort_unstable();
        unique_final_values.dedup();

        if unique_final_values.len() > 1 {
            return Err(format!(
                "got more than one final result: '{:?}'",
                final_values
            ));
        }

        Ok(())
    }
}
