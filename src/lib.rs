pub mod acceptor;
pub mod proposer;

#[derive(Clone)]
pub struct Msg {
    pub header: Header,
    pub body: Body,
}

#[derive(Clone)]
pub struct Header {
    pub from: Address,
    pub to: Address,
}

#[derive(Clone)]
pub enum Body {
    /// Request by an end-user.
    Request(Address),
    /// Response by a proposer to an end-user.
    Response,
    Prepare(Epoch),
    Promise(Option<Epoch>, Option<Epoch>, Option<Value>),
    Propose,
    Accept,
}

#[derive(Eq, Hash, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Address(String);

impl Address {
    pub fn new(a: &str) -> Address {
        Address(a.to_string())
    }
}

#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Instant(u64);

#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Epoch(u64);

#[derive(Clone, Default, PartialOrd, PartialEq)]
pub struct Value(String);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
