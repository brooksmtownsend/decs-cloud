use decscloud_common as codec;
use guest::prelude::*;

pub(crate) const SYSTEMS_KEY: &str = "decs:systems";

/// Stores a system in the KV store. Returns a boolean
/// that indicates whether the system already existed
pub(crate) fn put_system(
    ctx: &CapabilitiesContext,
    system: &codec::systemmgr::System,
) -> guest::wapc::Result<(bool, usize)> {
    let system_json = serde_json::to_string(system)?;
    let key = format!("system:{}", system.name);
    let existed = ctx.kv().exists(&key)?;

    if !existed {
        ctx.kv().list_add(SYSTEMS_KEY, &system.name)?;
    }
    ctx.kv().set(&key, &system_json, None)?;
    Ok((existed, index_of(ctx, SYSTEMS_KEY, &system.name)?))
}

pub(crate) fn get_systems(ctx: &CapabilitiesContext) -> Result<Vec<String>> {
    ctx.kv().list_range(SYSTEMS_KEY, 0, -1)
}

fn index_of(ctx: &CapabilitiesContext, listkey: &str, item: &str) -> Result<usize> {
    let members = ctx.kv().list_range(&listkey, 0, -1)?;
    Ok(members.iter().position(|s| *s == item).unwrap_or_else(|| 0))
}

pub(crate) fn get_system_list(
    ctx: &CapabilitiesContext,
    systems: Vec<String>,
) -> std::result::Result<Vec<codec::systemmgr::System>, Box<dyn std::error::Error>> {
    systems
        .iter()
        .filter_map(|sys_id| match get_system_details(ctx, sys_id) {
            Ok(r) => Some(Ok(r)),
            Err(_) => {
                // Item details have expired, remove it from the list
                ctx.kv().list_del_item(SYSTEMS_KEY, sys_id).unwrap();
                None
            }
        })
        .collect()
}

pub(crate) fn get_system_details(
    ctx: &CapabilitiesContext,
    system: &str,
) -> std::result::Result<codec::systemmgr::System, Box<dyn std::error::Error>> {
    let systemkey = format!("system:{}", system);

    if let Some(v) = ctx.kv().get(&systemkey)? {
        match serde_json::from_str::<codec::systemmgr::System>(&v) {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    } else {
        Err("system doesn't exist".into())
    }
}

/// Retrieves a list of all entities in a given shard that have a value for each
/// of the components indicated.
pub(crate) fn get_entities_for_component_set(
    ctx: &CapabilitiesContext,
    shard: &str,
    components: &[String],
) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
    let ek: Vec<String> = components
        .iter()
        .map(|c| format!("decs:{}:{}:entities", shard, c))
        .collect();
    ctx.kv().set_intersect(ek.as_slice()).map_err(|e| e.into())
}
