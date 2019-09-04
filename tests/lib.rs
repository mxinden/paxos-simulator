use paxos_simulator::acceptor::Acceptor;
use paxos_simulator::proposer::Proposer;
use paxos_simulator::{Body, Header, Msg};
use std::collections::{HashMap, VecDeque};

mod simulator;

#[test]
fn single_proposer_three_acceptors_one_request() {
    let mut p = HashMap::new();
    p.insert(
        "p1".to_string(),
        Proposer::new(vec!["a1".to_string(), "a2".to_string(), "a3".to_string()]),
    );

    let mut a = HashMap::new();
    a.insert("a1".to_string(), Acceptor::new());
    a.insert("a2".to_string(), Acceptor::new());
    a.insert("a3".to_string(), Acceptor::new());

    let mut inbox = VecDeque::new();
    inbox.push_back(Msg {
        header: Header {
            from: "u1".to_string(),
            to: "p1".to_string(),
        },
        body: Body::Request("v1".to_string()),
    });

    let mut s = simulator::Simulator::new(p, a, inbox);
    s.run().unwrap();

    assert_eq!(s.proposers.get("p1").unwrap().decided_value(), Some("v1".to_string()));
}
