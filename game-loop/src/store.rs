use guest::prelude::*;

const SHARDS_KEY: &str = "shards";
const MAX_SHARDS: isize = 1_000;

pub(crate) fn get_shards(ctx: &CapabilitiesContext) -> Result<Vec<String>> {
    ctx.kv().list_range(SHARDS_KEY, 0, MAX_SHARDS)
}
