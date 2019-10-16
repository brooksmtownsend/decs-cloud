use guest::prelude::*;

pub enum ComponentType {
    Collection,
    Model,
}

const TYPE_MODEL: &str = "M";
const TYPE_COLLECTION: &str = "C";

pub(crate) const NO_SUCH_COMPONENT: &str = "no such component";

/// Examines the type metadata for a given rid, returning whether it is a
/// model or a collection
pub(crate) fn component_type(ctx: &CapabilitiesContext, rid: &str) -> Result<ComponentType> {
    let key = format!("{}:type", rid.replace('.', ":"));
    let typeval = ctx.kv().get(&key)?;
    match typeval {
        Some(v) => {
            if v == TYPE_COLLECTION {
                Ok(ComponentType::Collection)
            } else {
                Ok(ComponentType::Model)
            }
        }
        None => Ok(ComponentType::Model),
    }
}

pub(crate) fn get_collection_rids(ctx: &CapabilitiesContext, rid: &str) -> Result<Vec<String>> {
    ctx.kv().list_range(&rid.replace('.', ":"), 0, -1)
}

/// Stores a single component value
pub(crate) fn put_component(ctx: &CapabilitiesContext, rid: &str, component: &str) -> Result<bool> {
    let tokens: Vec<&str> = rid.split('.').collect();
    let key = component_key(&tokens);
    let entkey = component_entities_key(&tokens);
    let typekey = format!("{}:type", key);

    let existed = ctx.kv().exists(&key)?;

    ctx.kv().set(&typekey, TYPE_MODEL, None)?;
    ctx.kv().set_add(&entkey, &entity_id(&tokens))?; // add entity to list of entities with a given component
    ctx.kv().set(&key, component, None)?;
    Ok(existed)
}

/// Adds a component value to the given collection
pub(crate) fn add_component_to_collection(
    ctx: &CapabilitiesContext,
    rid: &str,
    component: &str,
) -> Result<(usize, String)> {
    let tokens: Vec<&str> = rid.split('.').collect();
    let key = component_key(&tokens);
    let entkey = component_entities_key(&tokens);
    let typekey = format!("{}:type", key);
    let idkey = format!("{}:id", key);

    let id = ctx.kv().atomic_add(&idkey, 1)?;
    let new_rid = format!("{}.{}", rid, id);

    // add to the collection (the component key)
    let mut num_added = 0;
    ctx.kv().set(&typekey, TYPE_COLLECTION, None)?;
    if !ctx.kv().exists(&new_rid)? {
        num_added = ctx.kv().list_add(&key, &new_rid)?;
    }

    ctx.log(&format!("Adding '{}' to set '{}'", new_rid, key));

    // set the individual item
    let ridkey = new_rid.replace('.', ":");
    let ridtypekey = format!("{}:type", ridkey);
    ctx.kv().set(&ridkey, component, None)?;
    ctx.kv().set(&ridtypekey, TYPE_MODEL, None)?;

    ctx.kv().set_add(&entkey, &entity_id(&tokens))?; // add entity to the set of entities with a given component

    let members = ctx.kv().list_range(&key, 0, 1000)?;
    if num_added == 0 {
        Ok((
            members
                .iter()
                .position(|cmp| *cmp == rid)
                .unwrap_or_else(|| members.len() - 1),
            new_rid.clone(),
        ))
    } else {
        Ok((0, new_rid.clone())) // new item was added to the beginning
    }
}

fn index_of(ctx: &CapabilitiesContext, setkey: &str, item: &str) -> Result<usize> {
    let members = ctx.kv().list_range(&setkey, 0, -1)?;
    Ok(members.iter().position(|s| *s == item).unwrap_or_else(|| 0))
}

pub(crate) fn get_component(
    ctx: &CapabilitiesContext,
    rid: &str,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let key = rid.replace('.', ":");
    ctx.log(&format!("Querying single component: {}", key));
    match ctx.kv().get(&key) {
        Ok(v) => {
            if let Some(s) = v {
                Ok(s)
            } else {
                Err(NO_SUCH_COMPONENT.into())
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

pub(crate) fn delete_component(ctx: &CapabilitiesContext, rid: &str) -> Result<()> {
    let tokens: Vec<&str> = rid.split('.').collect();
    let key = rid.replace('.', ":");
    let type_key = format!("{}:type", key);
    let ent_key = component_entities_key(&tokens);

    ctx.kv().del_key(&type_key)?;
    ctx.kv().set_remove(&ent_key, &entity_id(&tokens))?;
    ctx.kv().del_key(&key)?;

    Ok(())
}

pub(crate) fn remove_component_from_collection(
    ctx: &CapabilitiesContext,
    rid: &str,
    item_rid: &str,
) -> Result<usize> {
    let key = rid.replace('.', ":");
    let item_key = item_rid.replace('.', ":");
    let item_type_key = format!("{}:type", item_key);

    ctx.log(&format!(
        "Attempting to remove {} from key {}",
        item_rid, key
    ));
    let idx = index_of(ctx, &key, item_rid)?;
    let remcount = ctx.kv().list_del_item(&key, item_rid)?;
    ctx.log(&format!("Removed {} items from list.", remcount));
    ctx.kv().del_key(&item_key)?;
    ctx.kv().del_key(&item_type_key)?;

    Ok(idx)
}

/// Extract the key-value store key for a single component from either the set or get RES protocol subject
pub(crate) fn component_key(tokens: &[&str]) -> String {
    tokens.join(":")
}

/// Extract the key-value store key for the set of entities which have a given
/// component associated with them.
/// decs:{shard}:{component}:entities
/// Subject looks like : call.decs.components.the_void.abc1234.position.set
pub(crate) fn component_entities_key(tokens: &[&str]) -> String {
    format!("decs:{}:{}:entities", tokens[2], tokens[4])
}

// decs.components.the_void.abc1234
pub(crate) fn entity_id(tokens: &[&str]) -> String {
    tokens[3].to_string()
}

#[cfg(test)]
mod test {
    use super::{component_entities_key, component_key};

    #[test]
    fn test_entities_key_extraction() {
        let subject = "decs.components.the_void.abc1234.position";
        let tokens: Vec<&str> = subject.split('.').collect();
        assert_eq!(
            "decs:the_void:position:entities",
            component_entities_key(&tokens)
        )
    }

    #[test]
    fn test_key_extraction() {
        let subject1 = "decs.components.the_void.abc1234.position";
        let subject3 = "decs.components.the_void.abc1234.radar_contacts";
        let tokens: Vec<&str> = subject1.split('.').collect();
        let tokens3: Vec<&str> = subject3.split('.').collect();

        assert_eq!(
            "decs:components:the_void:abc1234:position",
            component_key(&tokens)
        );

        assert_eq!(
            "decs:components:the_void:abc1234:radar_contacts",
            component_key(&tokens3)
        );
    }
}
