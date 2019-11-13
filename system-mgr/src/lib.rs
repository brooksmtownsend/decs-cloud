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

/// The system manager performs several main functions
/// 1. On appropriate intervals, publish a msg with the component value(s) on the topic `system.[shard].[system].frames`
/// 2. Periodically publishes messages on `system.registry`, to which systems must respond with their preferred frame rate and components of interest.
extern crate waxosuit_guest as guest;

#[macro_use]
extern crate serde_json;

use decscloud_common as codec;
use guest::prelude::*;

mod msg;
mod store;

call_handler!(handle_call);

pub fn handle_call(ctx: &CapabilitiesContext, operation: &str, msg: &[u8]) -> CallResult {
    match operation {
        codec::timer::OP_TIMER_TICK => msg::handle_timer(ctx, msg),
        messaging::OP_DELIVER_MESSAGE => msg::handle_message(ctx, msg),
        core::OP_HEALTH_REQUEST => Ok(vec![]),
        _ => Err("bad dispatch".into()),
    }
}
