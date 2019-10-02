use decscloud_codec as codec;
use guest::prelude::*;

const SHARDS_KEY: &str = "decs:shards";
const MAX_SHARDS: isize = 1_000;
pub(crate) const NOT_FOUND: &str = "Not found";

pub(crate) fn get_shards(ctx: &CapabilitiesContext) -> Result<Vec<String>> {
    ctx.kv().list_range(SHARDS_KEY, 0, MAX_SHARDS)
}

/// Creates or sets a shard. Returns a boolean indicating if the shard previously existed
pub(crate) fn put_shard(
    ctx: &CapabilitiesContext,
    shard: &codec::shard::Shard,
) -> std::result::Result<(usize, bool), Box<dyn std::error::Error>> {
    let new_count = ctx.kv().set_add(SHARDS_KEY, &shard.name)?;
    let existed = new_count == 0;
    let shard_key = format!("decs:shard:{}", shard.name);
    let shard_json = serde_json::to_string(shard)?;
    ctx.kv().set(&shard_key, &shard_json, None)?;

    let shards = ctx.kv().set_members(SHARDS_KEY)?;
    match shards.iter().position(|s| *s == shard.name) {
        Some(p) => Ok((p, existed)),
        None => Err("item not in set".into()),
    }
}

pub(crate) fn get_shard_details(
    ctx: &CapabilitiesContext,
    shard: &str,
) -> std::result::Result<codec::shard::Shard, Box<dyn std::error::Error>> {
    let shard_key = format!("shard:{}", shard);

    if let Some(v) = ctx.kv().get(&shard_key)? {
        match serde_json::from_str::<codec::shard::Shard>(&v) {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    } else {
        Err(NOT_FOUND.into())
    }
}
