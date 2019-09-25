use guest::prelude::*;

pub enum PutAction {
    None,
    CollectionAdd(usize),
    ModelChanged
}

pub(crate) const NO_SUCH_COMPONENT: &str = "no such component";

pub(crate) fn put_component(
    ctx: &CapabilitiesContext,
    tokens: &[&str],
    component: &str,
) -> Result<PutAction> {
    let key = component_key(tokens);
    let entkey = component_entities_key(tokens);

    let put_action = if ctx.kv().exists(&entkey)? {                    
        // model was in collection but changed, publish model change
        let existing = ctx.kv().get(&key)?.unwrap();
        if existing.trim() == component.trim() {
            PutAction::None // new and old components are the same
        } else {
            PutAction::ModelChanged 
        }        
    } else {
        let members = ctx.kv().set_members(&entkey)?;
        PutAction::CollectionAdd(members.len()) // new item in collection, publish collection add event
    };

    ctx.kv().set_add(&entkey, &entity_id(tokens))?;
    ctx.kv().set(&key, component, None)?;
    Ok(put_action)
}

pub(crate) fn get_component(
    ctx: &CapabilitiesContext,
    tokens: &[&str],
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let key = component_key(tokens);

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
    use super::{component_key, component_entities_key};

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
        let tokens: Vec<&str> = subject1.split('.').collect();
        let tokens2: Vec<&str> = subject2.split('.').collect();

        assert_eq!(
            "decs:components:the_void:abc1234:position",
            component_key(&tokens)
        );
        assert_eq!(
            "decs:components:the_void:abc1234:position",
            component_key(&tokens2)
        );
    }
}
