use decscloud_codec as codec;
use guest::prelude::*;

pub(crate) const SYSTEMS_KEY: &str = "decs:systems";

/// Stores a system in the KV store. Returns a boolean
/// that indicates whether the system already existed
pub(crate) fn put_system(
    ctx: &CapabilitiesContext,
    system: &codec::systemmgr::System,
) -> guest::wapc::Result<bool> {
    let system_json = serde_json::to_string(system)?;

    let new_count = ctx.kv().set_add(SYSTEMS_KEY, &system.name)?;
    ctx.kv()
        .set(&format!("system:{}", system.name), &system_json, None)?;
    Ok(new_count == 0)
}

pub(crate) fn get_systems(ctx: &CapabilitiesContext) -> Result<Vec<String>> {
    ctx.kv().set_members(SYSTEMS_KEY)
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
