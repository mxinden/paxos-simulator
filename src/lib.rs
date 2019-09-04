pub mod proposer;
pub mod acceptor;

pub struct Msg {
    header: Header,
    body: Body,
}

pub struct Header {
    from: Address,
    To: Address,
}

pub enum Body{
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
