pub mod inbound;
pub mod outbound;

#[allow(dead_code)]
pub enum Event {
    Inbound(inbound::InboundEvent),
    OutBound(outbound::OutBoundEvent)
}
