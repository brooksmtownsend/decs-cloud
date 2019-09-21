use decscloud_codec as codec;
use guest::prelude::*;

const SHARDS_KEY: &str = "shards";
const MAX_SHARDS: isize = 1_000;

pub(crate) fn get_shards(ctx: &CapabilitiesContext) -> Result<Vec<String>> {
    ctx.kv().list_range(SHARDS_KEY, 0, MAX_SHARDS)
}

pub(crate) fn add_shard(ctx: &CapabilitiesContext, shard: &codec::shard::Shard) -> Result<()> {
    ctx.kv().list_add(SHARDS_KEY, &shard.name)?;
    let shard_key = format!("shard:{}", shard.name);
    let shard_json = serde_json::to_string(shard)?;
    ctx.kv().set(&shard_key, &shard_json, None)?;
    Ok(())

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
        Err("shard doesn't exist".into())
    }
}
