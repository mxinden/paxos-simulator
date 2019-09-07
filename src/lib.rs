pub mod acceptor;
pub mod proposer;

#[derive(Clone, Debug, PartialEq)]
pub struct Msg {
    pub header: Header,
    pub body: Body,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub from: Address,
    pub to: Address,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    /// Request by an end-user.
    Request(Value),
    /// Response by a proposer to an end-user.
    Response(Value),
    Prepare(Epoch),
    /// Promised epoch, accepted epoch, accepted value.
    // TODO: Why not combine the two options, they never occur separately.
    Promise(Epoch, Option<Epoch>, Option<Value>),
    Propose(Epoch, Value),
    Accept(Epoch),
}

#[derive(Eq, Hash, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Address(String);

impl Address {
    pub fn new(a: &str) -> Address {
        Address(a.to_string())
    }
}

#[derive(Clone, Copy, Default, PartialOrd, PartialEq, Debug)]
pub struct Instant(pub u64);

impl std::ops::Add for Instant {
    type Output = Instant;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Add<u64> for Instant {
    type Output = Instant;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Clone, Copy, Default, PartialOrd, PartialEq, Debug)]
pub struct Epoch(u64);

#[derive(Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Value(String);

impl Value {
    pub fn new(v: &str) -> Self {
        Self(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
