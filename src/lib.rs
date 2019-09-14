use std::cmp::Ord;

pub mod acceptor;
pub mod proposer;

#[derive(Clone, PartialEq, Eq)]
pub struct Msg {
    pub header: Header,
    pub body: Body,
}

impl std::fmt::Debug for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.header, self.body)
    }
}

impl Ord for Msg {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.header.at.cmp(&other.header.at)
    }
}

impl PartialOrd for Msg {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Header {
    pub from: Address,
    pub to: Address,
    pub at: Instant,
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?} at {:?}", self.from, self.to, self.at)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Body {
    /// Request by an end-user.
    Request(Value),
    /// Response by a proposer to an end-user.
    Response(Value),
    Prepare(Epoch),
    /// Promised epoch, accepted epoch, accepted value.
    // TODO: Why not combine the two options, they never occur separately.
    Promise(Epoch, Option<(Epoch, Value)>),
    Propose(Epoch, Value),
    Accept(Epoch),
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Body::Request(v) => write!(f, "request({:?})", v),
            Body::Response(v) => write!(f, "response({:?})", v),
            Body::Prepare(e) => write!(f, "prepare({:?})", e),
            Body::Promise(e, a) => write!(f, "promise({:?}, {:?})", e, a),
            Body::Propose(e, v) => write!(f, "propose({:?}, {:?})", e, v),
            Body::Accept(e) => write!(f, "accept({:?})", e),
        }
    }
}

#[derive(Eq, Hash, Clone, Default, PartialOrd, PartialEq)]
pub struct Address(String);

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Address {
    pub fn new(a: &str) -> Address {
        Address(a.to_string())
    }
}

#[derive(Clone, Copy, Default, PartialOrd, PartialEq, Eq, Ord)]
pub struct Instant(pub u64);

impl std::fmt::Debug for Instant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

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

#[derive(Clone, Copy, Default, PartialOrd, PartialEq, Eq)]
pub struct Epoch(u64);

impl std::fmt::Debug for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Clone, Default, PartialOrd, PartialEq, Ord, Eq)]
pub struct Value(String);

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

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
