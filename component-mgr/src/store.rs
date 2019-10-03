use guest::prelude::*;

pub enum ComponentType {
    Collection,
    Model,
}

const TYPE_MODEL: &str = "M";
const TYPE_COLLECTION: &str = "C";

pub(crate) const NO_SUCH_COMPONENT: &str = "no such component";

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
    ctx.kv().set_members(&rid.replace('.', ":"))
}

pub(crate) fn put_component(
    ctx: &CapabilitiesContext,
    tokens: &[&str],
    component: &str,
) -> Result<()> {
    let key = component_key(tokens);
    let entkey = component_entities_key(tokens);
    let typekey = format!("{}:type", key);

    ctx.kv().set(&typekey, TYPE_MODEL, None)?;
    ctx.kv().set_add(&entkey, &entity_id(tokens))?; // add entity to list of entities with a given component
    ctx.kv().set(&key, component, None)?;
    Ok(())
}

pub(crate) fn add_component_to_collection(
    ctx: &CapabilitiesContext,
    tokens: &[&str],
    component: &str,
) -> Result<(usize, String)> {
    let key = component_key(tokens);
    let entkey = component_entities_key(tokens);
    let typekey = format!("{}:type", key);
    let idkey = format!("{}:id", key);

    let id = ctx.kv().atomic_add(&idkey, 1)?;

    let rid = format!("{}.{}", tokens[1..=5].join("."), id); // e.g. `decs.components.(shard).(entity).(collection-component).1`

    // add to the collection (the component key)
    ctx.kv().set(&typekey, TYPE_COLLECTION, None)?;
    let num_added = ctx.kv().set_add(&key, &rid)?;

    // set the individual item
    let ridkey = rid.replace('.', ":");
    let ridtypekey = format!("{}:type", ridkey);
    ctx.kv().set(&ridkey, component, None)?;
    ctx.kv().set(&ridtypekey, TYPE_MODEL, None)?;

    ctx.kv().set_add(&entkey, &entity_id(tokens))?; // add entity to list of entities with a given component
    let members = ctx.kv().set_members(&key)?;
    if num_added == 0 {
        Ok((
            members
                .iter()
                .position(|cmp| *cmp == rid)
                .unwrap_or_else(|| members.len() - 1),
            rid.clone(),
        ))
    } else {
        Ok((members.len() - 1, rid.clone())) // new item was added to the end
    }
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

/// Extract the key-value store key for a single component from either the set or get RES protocol subject
pub(crate) fn component_key(tokens: &[&str]) -> String {
    tokens[1..=5].join(":")
}

pub(crate) fn get_rid(tokens: &[&str]) -> String {
    if tokens[tokens.len() - 1] == "new" {
        tokens[1..=tokens.len() - 2].join(".")
    } else {
        tokens[1..tokens.len()].join(".")
    }
}
/// Extract the key-value store key for the set of entities which have a given
/// component associated with them.
/// decs:{shard}:{component}:entities
/// Subject looks like : call.decs.components.the_void.abc1234.position.set
pub(crate) fn component_entities_key(tokens: &[&str]) -> String {
    format!("decs:{}:{}:entities", tokens[3], tokens[5])
}

pub(crate) fn entity_id(tokens: &[&str]) -> String {
    tokens[4].to_string()
}

#[cfg(test)]
mod test {
    use super::{component_entities_key, component_key, get_rid};

    #[test]
    fn test_rid_extract() {
        let sub1 = "call.decs.components.the_void.player1.radar_contacts.new";
        let tokens1: Vec<&str> = sub1.split('.').collect();
        let sub2 = "get.decs.components.the_void.player1.radar_contacts.1";
        let tokens2: Vec<&str> = sub2.split('.').collect();

        assert_eq!(
            "decs.components.the_void.player1.radar_contacts",
            get_rid(&tokens1)
        );
        assert_eq!(
            "decs.components.the_void.player1.radar_contacts.1",
            get_rid(&tokens2)
        );
    }

    #[test]
    fn test_entities_key_extraction() {
        let subject = "call.decs.components.the_void.abc1234.position.set";
        let tokens: Vec<&str> = subject.split('.').collect();
        assert_eq!(
            "decs:the_void:position:entities",
            component_entities_key(&tokens)
        )
    }

    #[test]
    fn test_key_extraction() {
        let subject1 = "get.decs.components.the_void.abc1234.position";
        let subject2 = "call.decs.components.the_void.abc1234.position.set";
        let subject3 = "call.decs.components.the_void.abc1234.radar_contacts.new";
        let tokens: Vec<&str> = subject1.split('.').collect();
        let tokens2: Vec<&str> = subject2.split('.').collect();
        let tokens3: Vec<&str> = subject3.split('.').collect();

        assert_eq!(
            "decs:components:the_void:abc1234:position",
            component_key(&tokens)
        );
        assert_eq!(
            "decs:components:the_void:abc1234:position",
            component_key(&tokens2)
        );

        assert_eq!(
            "decs:components:the_void:abc1234:radar_contacts",
            component_key(&tokens3)
        );
    }
}
