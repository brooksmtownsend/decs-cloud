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
/// 1. On appropriate interval, publish a msg with the component value(s) on the topic `system.[shard].[system].frames`
/// 2. Periodically publish pings on `system.ping`, to which systems must respond with their preferred frame rate and components of interest.
/// 3. Respond to HTTP get requests on the `/systems` and `/system/[id]` URLs to allow querying the current state of the system registry
extern crate wascap_guest as guest;

use decscloud_codec as codec;
use guest::prelude::*;

mod msg;
mod rest;

call_handler!(handle_call);

pub fn handle_call(ctx: &CapabilitiesContext, operation: &str, msg: &[u8]) -> CallResult {
    match operation {
        http::OP_HANDLE_REQUEST => rest::handle_http(ctx, msg),
        codec::timer::OP_TIMER_TICK => msg::publish_frame(ctx, msg),
        messaging::OP_DELIVER_MESSAGE => msg::handle_message(ctx, msg),
        core::OP_HEALTH_REQUEST => Ok(vec![]),
        _ => Err("bad dispatch".into()),
    }
}
