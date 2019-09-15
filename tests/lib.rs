use paxos_simulator::{acceptor::Acceptor, proposer::Proposer, Address, Epoch, Instant, Value};
use paxos_simulator::{Body, Header, Msg};
use quickcheck::TestResult;
use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;

pub mod simulator;

#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[test]
fn single_proposer_three_acceptors_one_request() {
    let mut s = Builder::new()
        .with_proposers(1)
        .with_accpetors(3)
        .with_requests(vec![(1, 0)])
        .build();
    s.run().unwrap();
    s.ensure_correctness().unwrap();
}

#[test]
fn two_proposer_three_acceptors_two_request() {
    let mut s = Builder::new()
        .with_proposers(2)
        .with_accpetors(3)
        .with_requests(vec![(1, 0), (2, 1)])
        .build();
    s.run().unwrap();
    s.ensure_correctness().unwrap();
}

#[test]
fn regression_1() {
    let request_instants = vec![
        10, 64, 10, 64, 64, 10, 64, 10, 64, 10, 64, 64, 10, 10, 10, 10, 64, 6, 64,
    ];
    let mut rng = StdRng::seed_from_u64(0);
    let requests = request_instants
        .iter()
        .map(|i| (*i, rng.gen_range(0, 1)))
        .collect();

    let mut s = Builder::new()
        .with_proposers(1)
        .with_accpetors(3)
        .with_requests(requests)
        .build();
    s.run().unwrap();

    match s.ensure_correctness() {
        Ok(()) => {}
        Err(e) => {
            for l in s.log.iter() {
                println!("{}", l);
            }
            panic!(e);
        }
    }
}

#[quickcheck]
fn variable_requests(
    proposers: u32,
    acceptors: u32,
    request_instants: Vec<u64>,
    seed: u64,
) -> TestResult {
    if proposers == 0 || acceptors == 0 {
        return TestResult::discard();
    }

    if proposers > 10 || acceptors > 10 || request_instants.len() > 100 {
        return TestResult::discard();
    }

    let mut rng = StdRng::seed_from_u64(seed);

    let requests = request_instants
        .iter()
        .map(|i| (*i, rng.gen_range(0, proposers)))
        .collect();

    let mut simulator = Builder::new()
        .with_proposers(proposers)
        .with_accpetors(acceptors)
        .with_requests(requests)
        .build();

    simulator.run().unwrap();

    match simulator.ensure_correctness() {
        Ok(()) => TestResult::passed(),
        Err(e) => {
            for l in simulator.log.iter() {
                println!("{}", l);
            }
            TestResult::error(e)
        }
    }
}

#[derive(Default)]
pub struct Builder {
    a: HashMap<Address, Acceptor>,
    p: HashMap<Address, Proposer>,
    r: Vec<Msg>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder::default()
    }

    pub fn with_proposers(mut self, size: u32) -> Builder {
        for i in 0..size {
            let name = format!("p{}", i);

            self.p.insert(
                Address::new(&name),
                Proposer::new(Address::new(&name), Epoch::new(0, i), vec![]),
            );
        }

        self
    }

    pub fn with_accpetors(mut self, size: u32) -> Builder {
        for i in 0..size {
            let name = format!("a{}", i);

            self.a
                .insert(Address::new(&name), Acceptor::new(Address::new(&name)));
        }

        self
    }

    pub fn with_requests(mut self, r: Vec<(u64, u32)>) -> Builder {
        for (i, (instant, proposer)) in r.iter().enumerate() {
            let name = format!("p{}", proposer);
            let value = format!("v{}", i);

            self.r.push(Msg {
                header: Header {
                    from: Address::new("u1"),
                    to: Address::new(&name),
                    at: Instant(*instant),
                },
                body: Body::Request(Value::new(&value)),
            });
        }

        self
    }

    pub fn build(self) -> simulator::Simulator {
        let a_addresses: Vec<Address> = self.a.iter().map(|(_, a)| a.address()).collect();
        let p = self
            .p
            .into_iter()
            .map(|(address, mut proposer)| {
                proposer.acceptors = a_addresses.clone();
                (address, proposer)
            })
            .collect();

        simulator::Simulator::new(p, self.a, self.r)
    }
}
