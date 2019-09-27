use paxos_simulator::{Acceptor, Address, Body, Instant, Msg, Proposer, Value};
use rand::distributions::Distribution;
use std::collections::HashMap;

// Needs to be larger than proposer.rs/TIMEOUT.
const TIMEOUT: Instant = Instant(100);
const MAX_MSG_DELAY: Instant = Instant(5);

#[derive(Default, Debug)]
pub struct Simulator<A: Acceptor, P: Proposer, Rng: rand::Rng> {
    now: Instant,
    msg_delay_rng: Option<Rng>,

    proposers: HashMap<Address, P>,
    acceptors: HashMap<Address, A>,

    inbox: Vec<Msg>,
    /// Requests passed to the Simulator beforehand. Later used to ensure
    /// correctness of the simulation.
    requests: Vec<Msg>,
    responses: Vec<Msg>,

    // The simulator needs to be able to determine when the simulation is done,
    // thus not making any more progress. One could terminate once no messages
    // are being transferred anymore. But this would break proposer timeouts.
    // Instead let's wait a bit longer once there are no more messages.
    // `last_progress_at` is there to track the above.
    last_progress_at: Instant,

    /// Log lines collected to be printed on failure.
    pub log: Vec<String>,
}

impl<A: Acceptor, P: Proposer, Rng: rand::Rng> Simulator<A, P, Rng> {
    pub fn new(
        proposers: HashMap<Address, P>,
        acceptors: HashMap<Address, A>,
        requests: Vec<Msg>,
        msg_delay_rng: Option<Rng>,
    ) -> Simulator<A, P, Rng> {
        Simulator {
            now: Default::default(),
            msg_delay_rng,

            proposers,
            acceptors,

            // Init the inbox with the given requests.
            inbox: requests.clone(),
            requests,
            responses: vec![],

            last_progress_at: Default::default(),
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
            if self.inbox.is_empty() && self.now - self.last_progress_at > TIMEOUT {
                break;
            }

            // Safety measure to prevent infinite loops.
            if self.now > Instant(100_000) {
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
        if new_msgs.is_empty() {
            self.last_progress_at = self.now;
        }

        // Delay new messages if random number generator is set.
        if let Some(ref mut rng) = self.msg_delay_rng {
            new_msgs = new_msgs
                .into_iter()
                .map(|mut m| {
                    m.header.at = m.header.at + exp_distr_delay(rng);
                    m
                })
                .collect()
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

    /// Ensure that the past simulation is within the consistency guarantees we
    /// would like to achieve.
    ///
    /// Definition of consensus
    ///
    /// - All non-faulty processes eventually decide on a value.
    ///
    /// - All processes decide on the same value.
    ///
    /// - The decided value was intitially proposed.
    ///
    pub fn ensure_correctness(&self) -> Result<(), String> {
        println!("{:?}", self.responses);
        if self.responses.len() != self.requests.len() {
            return Err(format!(
                "expected {} responses, got {} responses",
                self.requests.len(),
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

        if self.requests.is_empty() {
            return Ok(());
        }

        let final_value = unique_final_values.get(0).unwrap();

        let mut decided_value_initialy_proposed = false;
        for req in self.requests.iter() {
            match &req.body {
                Body::Request(v) => {
                    if v == final_value {
                        decided_value_initialy_proposed = true;
                    }
                }
                _ => unreachable!(),
            }
        }

        if !decided_value_initialy_proposed {
            return Err(format!(
                "expected decided value to be among the initially proposed
                values, got value \"{:?}\", initial requests \"{:?}\"",
                final_value, self.requests,
            ));
        }

        Ok(())
    }
}

/// Returns an emulated network delay based on an exponential distribution.
fn exp_distr_delay<Rng: rand::Rng>(rng: &mut Rng) -> Instant {
    std::cmp::min(
        Instant(
            rand_distr::Float::to_u64(
                // Choosing 0.5 is not backed by anything more than trial and error.
                rand_distr::Exp::new(0.5).unwrap().sample(rng),
            )
            .unwrap(),
        ),
        MAX_MSG_DELAY,
    )
}
