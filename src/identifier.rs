use std::fmt::Display;

const DIVIDER: &'static str = ":";
const DEFAULT_ID: &'static str = "unknown";

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Identifier<const DEFAULT_NAMESPACE: &'static str = "game"> {
    scope: String,
    id: String,
}

impl Identifier {
    pub fn scope(&self) -> &String {
        &self.scope
    }

    pub fn id(&self) -> &String {
        &self.id
    }
}

impl<const DEFAULT_NAMESPACE: &'static str>  From<String> for Identifier<DEFAULT_NAMESPACE> {
    fn from(value: String) -> Self {
        let cleaned = if !value.starts_with(DIVIDER) {
            value
        } else {
            value.replacen(DIVIDER, "", 1)
        };
        if cleaned.contains(DIVIDER) {
            let split: Vec<&str> = cleaned.split(DIVIDER).collect();
            let scope: String = (*split.get(0).unwrap()).into();
            let id: String = split[1..].join("_");
            Self {
                scope,
                id: if !id.eq("") { id } else { DEFAULT_ID.into() },
            }
        } else {
            Self {
                scope: DEFAULT_NAMESPACE.into(),
                id: cleaned,
            }
        }
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self::from(String::from(value))
    }
}

impl Into<String> for Identifier {
    fn into(self) -> String {
        self.scope + &String::from(":") + &self.id
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.scope)?;
        f.write_str(DIVIDER)?;
        f.write_str(&self.id)
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::identifier::DEFAULT_ID;

    use super::{Identifier, DIVIDER};
    
    const NAMESPACE: &'static str = "namespace";

    #[test]
    fn from() {
        let scope = "scope";
        let id = "id";
        let identifier: Identifier<NAMESPACE> = Identifier::from(vec![scope, DIVIDER.into(), id].concat());

        assert_eq!(identifier.scope, scope);
        assert_eq!(identifier.id, id);
    }

    #[test]
    fn from_no_scope() {
        let id = "id";
        let identifier: Identifier<NAMESPACE> = Identifier::from(String::from(id));

        assert_eq!(identifier.scope, NAMESPACE);
        assert_eq!(identifier.id, id);
    }

    #[test]
    fn from_dirty_no_scope() {
        let id = "id";
        let identifier: Identifier<NAMESPACE> = Identifier::from(vec![DIVIDER, id].concat());

        assert_eq!(identifier.scope, NAMESPACE);
        assert_eq!(identifier.id, id);
    }

    #[test]
    fn from_no_id() {
        let scope = "scope";
        let identifier: Identifier = Identifier::from(vec![scope, DIVIDER].concat());

        assert_eq!(identifier.scope, scope);
        assert_eq!(identifier.id, DEFAULT_ID);
    }

    #[test]
    fn hashable() {
        let mut map: HashMap<Identifier, bool> = HashMap::default();
        map.insert(Identifier::from("id"), true);

        assert!(map.contains_key(&Identifier::from("id")))
    }
}
