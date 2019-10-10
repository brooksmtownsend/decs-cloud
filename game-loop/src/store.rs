use guest::prelude::*;

const SHARDS_KEY: &str = "decs:shards";


// TODO: game loop should not be tightly coupled to the shard schema. It should
// instead make a request of the shard manager to get a list of shards. We should
// evaluate this once we have benchmarking capabilities to see how it might impact
// throughput.
pub(crate) fn get_shards(ctx: &CapabilitiesContext) -> Result<Vec<String>> {
    ctx.kv().set_members(SHARDS_KEY)
}
