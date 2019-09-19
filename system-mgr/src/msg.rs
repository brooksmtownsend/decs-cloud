use crate::store;
use decscloud_codec as codec;
use guest::prelude::*;

const PING_EVERY_TICKS: i64 = 200;
const REGISTRY_PING_SUBJECT: &str = "system.registry";
const REGISTRY_PONG_SUBJECT: &str = "system.registry.replies";

pub fn handle_timer(
    ctx: &CapabilitiesContext,
    tick: impl Into<codec::timer::TimerTick>,
) -> CallResult {
    let tick = tick.into();
    if tick.seq_no % PING_EVERY_TICKS == 0 {
        emit_ping(ctx)
    } else {
        // publish_frame(...)
        Ok(vec![])
    }
}

fn emit_ping(ctx: &CapabilitiesContext) -> CallResult {
    ctx.log("Emitting registration ping");
    ctx.msg().publish(REGISTRY_PING_SUBJECT, Some(REGISTRY_PONG_SUBJECT), &[])?;
    Ok(vec![])
}

/*
/// Publishes 1 frame per entity on each registered system's inbound
/// frame topic `system.[shard].[system-name].frames`
pub fn publish_frame(
    _ctx: &CapabilitiesContext,
    _tick: impl Into<codec::timer::TimerTick>,
) -> CallResult {
    Ok(vec![])
}
*/

// TODO - every time we get a registration back from a system, check all
// the other ones for expiration and purge them

/// The system manager should subscribe to a topic like `system.registry.replies`, which is where all
/// live systems will continually post their data in order for the system manager to dispense
/// frames to them. We don't need to do a branch on the topic name here because the only subscription
/// a system manager should have is to the `system.registry.replies` topic.
pub fn handle_message(
    ctx: &CapabilitiesContext,
    msg: impl Into<messaging::DeliverMessage>,
) -> CallResult {
    let msg = msg.into().message;
    if let Some(msg) = msg {
        let system: codec::systemmgr::System = serde_json::from_slice(&msg.body)?;
        store::put_system(ctx, &system)?;
        Ok(vec![])
    } else {
        Err("no message payload on subject".into())
    }
}
