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

//! # Component Manager
//!
//! The component manager exposes a RES-protocol compliant interface to allow
//! for the querying and manipulation of the following resources:
//!
//! decs.components.{shard-id}.{entity-id}.{component-name} - get/set
//! decs.components.{shard-id}.{entity-id} - get (collection)
//!
extern crate decscloud_codec as decs;
extern crate waxosuit_guest as guest;

use guest::prelude::*;

#[macro_use]
extern crate serde_json;

call_handler!(handle_call);

pub fn handle_call(ctx: &CapabilitiesContext, operation: &str, msg: &[u8]) -> CallResult {
    match operation {
        messaging::OP_DELIVER_MESSAGE => msg::handle_message(ctx, msg),
        core::OP_HEALTH_REQUEST => Ok(vec![]),
        _ => Err("bad dispatch".into()),
    }
}

mod msg;
mod store;
