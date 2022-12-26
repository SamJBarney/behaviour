use std::fmt::Display;

const DEFAULT_NAMESPACE: &'static str = "game";
const DIVIDER: &'static str = ":";
const DEFAULT_ID: &'static str = "unknown";

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Identifier {
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

impl From<String> for Identifier {
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

    use crate::identifier::{DEFAULT_ID, DEFAULT_NAMESPACE};

    use super::{Identifier, DIVIDER};

    #[test]
    fn from() {
        let scope = "scope";
        let id = "id";
        let identifier = Identifier::from(vec![scope, DIVIDER.into(), id].concat());

        assert_eq!(identifier.scope, scope);
        assert_eq!(identifier.id, id);
    }

    #[test]
    fn from_no_scope() {
        let id = "id";
        let identifier = Identifier::from(String::from(id));

        assert_eq!(identifier.scope, DEFAULT_NAMESPACE);
        assert_eq!(identifier.id, id);
    }

    #[test]
    fn from_dirty_no_scope() {
        let id = "id";
        let identifier = Identifier::from(vec![DIVIDER, id].concat());

        assert_eq!(identifier.scope, DEFAULT_NAMESPACE);
        assert_eq!(identifier.id, id);
    }

    #[test]
    fn from_no_id() {
        let scope = "scope";
        let identifier = Identifier::from(vec![scope, DIVIDER].concat());

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
