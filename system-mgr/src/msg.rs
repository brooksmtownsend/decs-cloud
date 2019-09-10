use decscloud_codec as codec;
use guest::prelude::*;

/// Publishes 1 frame per entity on each registered system's inbound
/// frame topic `system.[shard].[system-name].frames`
pub fn publish_frame(
    _ctx: &CapabilitiesContext,
    _tick: impl Into<codec::timer::TimerTick>,
) -> CallResult {
    Ok(vec![])
}

// TODO - every time we get a registration back from a system, check all
// the other ones for expiration and purge them

/// The system manager should subscribe to a topic like `system.registry`, which is where all
/// live systems will continually post their data in order for the system manager to dispense
/// frames to them
pub fn handle_message(
    ctx: &CapabilitiesContext,
    msg: impl Into<messaging::DeliverMessage>,
) -> CallResult {
    let msg = msg.into().message;
    if let Some(msg) = msg {
        let system: codec::systemmgr::System = serde_json::from_slice(&msg.body)?;
        crate::rest::put_system(ctx, &system)?;
        Ok(vec![])
    } else {
        Err("no message payload on subject".into())
    }
}
