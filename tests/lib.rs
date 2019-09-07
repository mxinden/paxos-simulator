use paxos_simulator::{acceptor::Acceptor, proposer::Proposer, Address, Value};
use paxos_simulator::{Body, Header, Msg};
use std::collections::{HashMap, VecDeque};

mod simulator;

#[test]
fn single_proposer_three_acceptors_one_request() {
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
    inbox.push_back(Msg {
        header: Header {
            from: Address::new("u1"),
            to: Address::new("p1"),
        },
        body: Body::Request(Value::new("v1")),
    });

    let mut s = simulator::Simulator::new(p, a, inbox);
    s.run().unwrap();

    // TODO: Check if we got any responses.
}
