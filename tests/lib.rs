use paxos_simulator::{acceptor::Acceptor, proposer::Proposer, Address, Instant, Value};
use paxos_simulator::{Body, Header, Msg};
use quickcheck::TestResult;
use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};
use std::collections::{HashMap, VecDeque};

pub mod simulator;

#[macro_use(quickcheck)]
extern crate quickcheck_macros;

// #[test]
// fn single_proposer_three_acceptors_one_request() {
//     let mut p = HashMap::new();
//     p.insert(
//         Address::new("p1"),
//         Proposer::new(
//             Address::new("p1"),
//             vec![Address::new("a1"), Address::new("a2"), Address::new("a3")],
//         ),
//     );
//
//     let mut a = HashMap::new();
//     a.insert(Address::new("a1"), Acceptor::new());
//     a.insert(Address::new("a2"), Acceptor::new());
//     a.insert(Address::new("a3"), Acceptor::new());
//
//     let mut inbox = VecDeque::new();
//     inbox.push_back(Msg {
//         header: Header {
//             from: Address::new("u1"),
//             to: Address::new("p1"),
//             at: Instant(1),
//         },
//         body: Body::Request(Value::new("v1")),
//     });
//
//     let mut s = simulator::Simulator::new(p, a, inbox);
//     s.run().unwrap();
//
//     assert_eq!(s.responses.len(), 1);
//     assert_eq!(s.responses[0], Msg {
//         header: Header {
//             from: Address::new("p1"),
//             // TODO: We need to track the client address along the way.
//             to: Address::new(""),
//             at: Instant(5),
//         },
//         body: Body::Response(Value::new("v1")),
//     })
// }

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

    if simulator.responses.len() != request_instants.len() {
        return TestResult::error(format!(
            "expected {} responses, got {} responses",
            simulator.responses.len(),
            request_instants.len()
        ));
    }

    let final_values = simulator.responses.into_iter().map(|r| match r.body {
        Body::Response(v) => v,
        _ => unreachable!(),
    }).collect::<Vec<Value>>();

    println!("{:?}", final_values);

    let mut unique_final_values = final_values.clone();
    unique_final_values.sort_unstable();
    unique_final_values.dedup();

    if unique_final_values.len() > 1 {
        return TestResult::error(format!("got more than one final result: '{:?}'", final_values));
    }

    TestResult::passed()
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
