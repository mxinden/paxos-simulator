use crate::{Epoch, Value};

pub use acceptor::Acceptor;
pub use proposer::Proposer;

mod acceptor;
mod proposer;

#[derive(Clone, PartialEq, Eq)]
pub enum Body {
    /// Request by an end-user.
    Request(Value),
    /// Response by a proposer to an end-user.
    Response(Value),
    Prepare(Epoch),
    /// Promised epoch, accepted epoch, accepted value.
    Promise(Epoch, Option<(Epoch, Value)>),
    Propose(Epoch, Value),
    Accept(Epoch),
    /// Negative acknowledgment of either a Prepare or a Propose message send by
    /// an Acceptor to a Proposer. First epoch represents the declined one,
    /// second represents the highest known one.
    Nack(Epoch, Epoch),
}

impl crate::Body for Body {
    fn is_request(&self) -> Option<Value> {
        if let Body::Request(v) = self {
            return Some(v.clone());
        }

        None
    }

    fn is_response(&self) -> Option<Value> {
        if let Body::Response(v) = self {
            return Some(v.clone());
        }

        None
    }
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
            Body::Nack(e, e_high) => write!(f, "nack({:?}, {:?})", e, e_high),
        }
    }
}
