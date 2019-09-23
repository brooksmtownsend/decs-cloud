use guest::prelude::*;

pub(crate) const NO_SUCH_COMPONENT: &str = "no such component";

pub(crate) fn put_component(
    ctx: &CapabilitiesContext,
    tokens: &[&str],
    component: &str,
) -> Result<()> {
    let key = component_key(tokens);
    let _entkey = component_entities_key(tokens);

    // ctx.kv().set_add(entkey, entity_id(tokens))?;
    ctx.kv().set(&key, component, None)
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
fn component_key(tokens: &[&str]) -> String {
    tokens[1..=5].join(":")
}

/// Extract the key-value store key for the set of entities which have a given
/// component associated with them.
/// decs:{shard}:{component}:entities
/// Subject looks like : call.decs.components.the_void.abc1234.position.set
fn component_entities_key(tokens: &[&str]) -> String {    
    format!("decs:{}:{}:entities", tokens[3], tokens[5])
}

/*
 TODO - uncomment this after we have set operations in the KV store
fn entity_id(tokens: &[&str]) -> String {
    tokens[4].to_string()
} */


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