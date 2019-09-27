use paxos_simulator::{
    classic, nack, Acceptor, Address, Body, Epoch, Header, Instant, Msg, Proposer, Value,
};
use quickcheck::TestResult;
use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};
use std::collections::HashMap;

pub mod simulator;

#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[test]
fn single_proposer_three_acceptors_one_request() {
    let mut s = ClassicPaxosBuilder::<StdRng>::new()
        .with_proposers(1)
        .with_accpetors(3)
        .with_requests(vec![(1, 0)])
        .build();
    s.run().unwrap();
    s.ensure_correctness().unwrap();
}

#[test]
fn two_proposer_three_acceptors_two_request() {
    let mut s = ClassicPaxosBuilder::<StdRng>::new()
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

    let mut s = ClassicPaxosBuilder::<StdRng>::new()
        .with_proposers(1)
        .with_accpetors(3)
        .with_requests(requests)
        .with_msg_delay_rng(rng)
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

// TODO: Find a way to abstract over classic and nack to not need to duplicate
// the code below.
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

    let mut simulator = ClassicPaxosBuilder::<StdRng>::new()
        .with_proposers(proposers)
        .with_accpetors(acceptors)
        .with_requests(requests)
        .with_msg_delay_rng(rng)
        .build();

    simulator.run().unwrap();

    match simulator.ensure_correctness() {
        Ok(()) => (),
        Err(e) => {
            for l in simulator.log.iter() {
                println!("{}", l);
            }
            return TestResult::error(e);
        }
    };

    let classic_duration = simulator.get_now();

    let mut rng = StdRng::seed_from_u64(seed);

    let requests = request_instants
        .iter()
        .map(|i| (*i, rng.gen_range(0, proposers)))
        .collect();

    let mut simulator = NackPaxosBuilder::<StdRng>::new()
        .with_proposers(proposers)
        .with_accpetors(acceptors)
        .with_requests(requests)
        .with_msg_delay_rng(rng)
        .build();

    simulator.run().unwrap();

    match simulator.ensure_correctness() {
        Ok(()) => (),
        Err(e) => {
            for l in simulator.log.iter() {
                println!("{}", l);
            }
            return TestResult::error(e);
        }
    }

    let nack_duration = simulator.get_now();

    println!("classic: {:?}, nack: {:?}", classic_duration, nack_duration);

    TestResult::passed()
}

trait Builder<A: Acceptor<B>, P: Proposer<B>, B: Body, Rng: rand::Rng> {
    fn with_proposers(self, size: u32) -> Self;
    fn with_accpetors(self, size: u32) -> Self;
    fn with_requests(self, r: Vec<(u64, u32)>) -> Self;
    fn with_msg_delay_rng(self, rng: Rng) -> Self;

    fn build(self) -> simulator::Simulator<A, P, B, Rng>;
}

#[derive(Default)]
struct ClassicPaxosBuilder<Rng: rand::Rng> {
    a: HashMap<Address, classic::Acceptor>,
    p: HashMap<Address, classic::Proposer>,
    r: Vec<Msg<classic::Body>>,
    msg_delay_rng: Option<Rng>,
}

impl<Rng: rand::Rng> ClassicPaxosBuilder<Rng> {
    fn new() -> Self {
        ClassicPaxosBuilder {
            a: Default::default(),
            p: Default::default(),
            r: Default::default(),
            msg_delay_rng: None,
        }
    }
}

impl<Rng: rand::Rng> Builder<classic::Acceptor, classic::Proposer, classic::Body, Rng>
    for ClassicPaxosBuilder<Rng>
{
    fn with_proposers(mut self, size: u32) -> Self {
        for i in 0..size {
            let name = format!("p{}", i);

            self.p.insert(
                Address::new(&name),
                classic::Proposer::new(Address::new(&name), Epoch::new(0, i), vec![]),
            );
        }

        self
    }

    fn with_accpetors(mut self, size: u32) -> Self {
        for i in 0..size {
            let name = format!("a{}", i);

            self.a.insert(
                Address::new(&name),
                classic::Acceptor::new(Address::new(&name)),
            );
        }

        self
    }

    fn with_requests(mut self, r: Vec<(u64, u32)>) -> Self {
        for (i, (instant, proposer)) in r.iter().enumerate() {
            let name = format!("p{}", proposer);
            let value = format!("v{}", i);

            self.r.push(Msg {
                header: Header {
                    from: Address::new("u1"),
                    to: Address::new(&name),
                    at: Instant(*instant),
                },
                body: classic::Body::Request(Value::new(&value)),
            });
        }

        self
    }

    fn with_msg_delay_rng(mut self, rng: Rng) -> Self {
        self.msg_delay_rng = Some(rng);
        self
    }

    fn build(
        self,
    ) -> simulator::Simulator<classic::Acceptor, classic::Proposer, classic::Body, Rng> {
        let a_addresses: Vec<Address> = self.a.iter().map(|(_, a)| a.address()).collect();
        let p = self
            .p
            .into_iter()
            .map(|(address, mut proposer)| {
                proposer.acceptors = a_addresses.clone();
                (address, proposer)
            })
            .collect();

        simulator::Simulator::new(p, self.a, self.r, self.msg_delay_rng)
    }
}

#[derive(Default)]
struct NackPaxosBuilder<Rng: rand::Rng> {
    a: HashMap<Address, nack::Acceptor>,
    p: HashMap<Address, nack::Proposer>,
    r: Vec<Msg<nack::Body>>,
    msg_delay_rng: Option<Rng>,
}

impl<Rng: rand::Rng> NackPaxosBuilder<Rng> {
    fn new() -> Self {
        NackPaxosBuilder {
            a: Default::default(),
            p: Default::default(),
            r: Default::default(),
            msg_delay_rng: None,
        }
    }
}

impl<Rng: rand::Rng> Builder<nack::Acceptor, nack::Proposer, nack::Body, Rng>
    for NackPaxosBuilder<Rng>
{
    fn with_proposers(mut self, size: u32) -> Self {
        for i in 0..size {
            let name = format!("p{}", i);

            self.p.insert(
                Address::new(&name),
                nack::Proposer::new(Address::new(&name), Epoch::new(0, i), vec![]),
            );
        }

        self
    }

    fn with_accpetors(mut self, size: u32) -> Self {
        for i in 0..size {
            let name = format!("a{}", i);

            self.a.insert(
                Address::new(&name),
                nack::Acceptor::new(Address::new(&name)),
            );
        }

        self
    }

    fn with_requests(mut self, r: Vec<(u64, u32)>) -> Self {
        for (i, (instant, proposer)) in r.iter().enumerate() {
            let name = format!("p{}", proposer);
            let value = format!("v{}", i);

            self.r.push(Msg {
                header: Header {
                    from: Address::new("u1"),
                    to: Address::new(&name),
                    at: Instant(*instant),
                },
                body: nack::Body::Request(Value::new(&value)),
            });
        }

        self
    }

    fn with_msg_delay_rng(mut self, rng: Rng) -> Self {
        self.msg_delay_rng = Some(rng);
        self
    }

    fn build(self) -> simulator::Simulator<nack::Acceptor, nack::Proposer, nack::Body, Rng> {
        let a_addresses: Vec<Address> = self.a.iter().map(|(_, a)| a.address()).collect();
        let p = self
            .p
            .into_iter()
            .map(|(address, mut proposer)| {
                proposer.acceptors = a_addresses.clone();
                (address, proposer)
            })
            .collect();

        simulator::Simulator::new(p, self.a, self.r, self.msg_delay_rng)
    }
}
