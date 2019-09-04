mod simulator;

#[test]
fn basic() {
    simulator::Simulator::new().run().unwrap();
}
