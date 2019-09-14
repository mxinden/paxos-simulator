use paxos_simulator::{acceptor::Acceptor, proposer::Proposer, Address, Instant, Value};
use paxos_simulator::{Body, Header, Msg};
use quickcheck::TestResult;
use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};
use std::collections::{HashMap, VecDeque};

pub mod simulator;

#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[test]
fn single_proposer_three_acceptors_one_request() {
    let mut s = Builder::new().with_proposers(1).with_accpetors(3).with_requests(vec![(1,0)]).build();

    s.run().unwrap();

    s.ensure_correctness().unwrap();
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
    if proposers > 5 || acceptors > 5 || request_instants.len() > 5 {
        return TestResult::discard();
    }

    println!(
        "test with {}, {}, {:?}",
        proposers, acceptors, request_instants
    );

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
        Err(e) => TestResult::error(e),
    }
}

#[derive(Default)]
pub struct Builder {
    a: HashMap<Address, Acceptor>,
    p: HashMap<Address, Proposer>,
    r: VecDeque<Msg>,
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
                Proposer::new(Address::new(&name), vec![]),
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

            self.r.push_back(Msg {
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
