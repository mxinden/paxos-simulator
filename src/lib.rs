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
    Request(String),
    Prepare(Instant),
    Promise(Option<Instant>, Option<Instant>, Option<Value>),
    Propose,
    Accept,
}

pub type Address = String;

// TODO: Rename to epoch?
pub type Instant = u64;

pub type Value = String;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
