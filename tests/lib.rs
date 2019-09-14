use paxos_simulator::{acceptor::Acceptor, proposer::Proposer, Address, Value, Instant};
use paxos_simulator::{Body, Header, Msg};
use std::collections::{HashMap, VecDeque};
use quickcheck::TestResult;

mod simulator;

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
fn variable_requests(request_instants: Vec<u64>) -> TestResult {
    if request_instants.len() > 50 {
        return TestResult::discard();
    }

    let mut p = HashMap::new();
    p.insert(
        Address::new("p1"),
        Proposer::new(
            Address::new("p1"),
            vec![Address::new("a1"), Address::new("a2"), Address::new("a3")],
        ),
    );

    let mut a = HashMap::new();
    a.insert(Address::new("a1"), Acceptor::new());
    a.insert(Address::new("a2"), Acceptor::new());
    a.insert(Address::new("a3"), Acceptor::new());

    let mut inbox = VecDeque::new();
    for instant in request_instants.iter() {
        inbox.push_back(Msg {
            header: Header {
                from: Address::new("u1"),
                to: Address::new("p1"),
                at: Instant(*instant),
            },
            body: Body::Request(Value::new("v1")),
        });
    }

    let mut s = simulator::Simulator::new(p, a, inbox);
    s.run().unwrap();

    if s.responses.len() != request_instants.len() {
        return TestResult::error(format!("expected {} responses, got {} responses", s.responses.len(), request_instants.len()));
    }

    for r in s.responses.iter() {
        if r.body != Body::Response(Value::new("v1")) {
            return TestResult::error(format!("expected 'v1' response, got '{:?}' body", r.body));
        }
    }

    TestResult::passed()
}
