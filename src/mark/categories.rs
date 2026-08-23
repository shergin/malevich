//! Compact storage for a categorical mark channel.

use std::collections::HashMap;

/// Distinct labels in first-appearance order and one label index per datum.
///
/// Mark constructors intern the incoming strings once. Resolution can then use
/// stable integer identities without rediscovering categories or retaining one
/// owned string per datum.
#[derive(Clone)]
pub(crate) struct Categories {
    labels: Vec<String>,
    ids: Vec<usize>,
}

impl Categories {
    pub(crate) fn new(values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let values = values.into_iter();
        let (lower, _) = values.size_hint();
        let mut lookup = HashMap::with_capacity(lower);
        let mut ids = Vec::with_capacity(lower);

        for value in values {
            let label = value.into();
            let next = lookup.len();
            let id = *lookup.entry(label).or_insert(next);
            ids.push(id);
        }

        let mut labels = vec![String::new(); lookup.len()];
        for (label, id) in lookup {
            labels[id] = label;
        }
        Self { labels, ids }
    }

    pub(crate) fn len(&self) -> usize {
        self.ids.len()
    }

    pub(crate) fn labels(&self) -> &[String] {
        &self.labels
    }

    pub(crate) fn ids(&self) -> &[usize] {
        &self.ids
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::ser::SerializeSeq as _;

    use super::Categories;

    impl serde::Serialize for Categories {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut sequence = serializer.serialize_seq(Some(self.ids.len()))?;
            for &id in &self.ids {
                sequence.serialize_element(&self.labels[id])?;
            }
            sequence.end()
        }
    }

    impl<'de> serde::Deserialize<'de> for Categories {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Vec::<String>::deserialize(deserializer).map(Categories::new)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Categories;

    #[test]
    fn interns_labels_in_first_appearance_order() {
        let categories = Categories::new(["b", "a", "b", "c", "a"]);
        assert_eq!(categories.labels(), ["b", "a", "c"]);
        assert_eq!(categories.ids(), [0, 1, 0, 2, 1]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_keeps_the_expanded_wire_representation() {
        let categories = Categories::new(["b", "a", "b"]);
        let json = serde_json::to_string(&categories).expect("serializes");
        assert_eq!(json, r#"["b","a","b"]"#);

        let decoded: Categories = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(decoded.labels(), ["b", "a"]);
        assert_eq!(decoded.ids(), [0, 1, 0]);
    }
}
