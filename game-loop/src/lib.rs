// Copyright 2015-2019 Capital One Services, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate waxosuit_guest as guest;

use decscloud_codec as decs;
use guest::prelude::*;

call_handler!(handle_call);

pub fn handle_call(ctx: &CapabilitiesContext, operation: &str, msg: &[u8]) -> CallResult {
    match operation {
        decs::timer::OP_TIMER_TICK => tick(ctx, msg),
        core::OP_HEALTH_REQUEST => Ok(vec![]),
        _ => Err("bad dispatch".into()),
    }
}

/// Every time the game loop ticks, publish a "loop tick" on decs.(shard).gameloop
/// This allows all system managers to receive distributed loop ticks, and can allow
/// a single system manager to subscribe to ticks for a single shard
fn tick(ctx: &CapabilitiesContext, tick: impl Into<decs::timer::TimerTick>) -> CallResult {
    let tick = tick.into();
    let shards = store::get_shards(ctx)?;

    for shard in shards.iter() {
        let gtick = decs::timer::GameLoopTick::from_tick(&tick, &shard);
        ctx.msg().publish(
            &format!("decs.{}.gameloop", shard),
            None,
            &serde_json::to_vec(&gtick)?,
        )?;
    }

    Ok(vec![])
}

mod store;
