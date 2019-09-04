pub mod proposer;
pub mod acceptor;

pub struct Msg {
    pub header: Header,
    pub body: Body,
}

pub struct Header {
    pub from: Address,
    pub to: Address,
}

pub enum Body{
    Request(String),
    Prepare,
    Promise,
    Propose,
    Accept,
}

pub type Address = String;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
