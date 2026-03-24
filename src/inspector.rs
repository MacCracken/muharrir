//! Generic property inspector — gather and display component/property info.
//!
//! Provides a uniform way to present object properties in an editor panel,
//! regardless of the domain (game entities, image layers, audio tracks).

use serde::{Deserialize, Serialize};

/// A single property for display in the inspector panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Property {
    /// Category or component name (e.g. "Transform", "Material", "Audio").
    pub category: &'static str,
    /// Property name within the category (e.g. "position", "color", "gain").
    pub name: &'static str,
    /// Human-readable value string.
    pub value: String,
}

impl Property {
    /// Create a new property.
    #[must_use]
    #[inline]
    pub fn new(category: &'static str, name: &'static str, value: impl Into<String>) -> Self {
        Self {
            category,
            name,
            value: value.into(),
        }
    }
}

/// A collection of properties for a selected object.
#[derive(Debug, Clone, Default)]
pub struct PropertySheet {
    /// All properties gathered for the selected object.
    pub properties: Vec<Property>,
}

impl PropertySheet {
    /// Create a new empty property sheet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a property to the sheet.
    pub fn push(&mut self, property: Property) {
        self.properties.push(property);
    }

    /// Number of properties.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Whether the sheet is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    /// Get properties filtered by category.
    #[must_use]
    #[inline]
    pub fn by_category(&self, category: &str) -> Vec<&Property> {
        self.properties
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get unique category names in order of first appearance.
    #[must_use]
    pub fn categories(&self) -> Vec<&'static str> {
        let mut seen = Vec::new();
        for p in &self.properties {
            if !seen.contains(&p.category) {
                seen.push(p.category);
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_new() {
        let p = Property::new("Transform", "position", "(1, 2, 3)");
        assert_eq!(p.category, "Transform");
        assert_eq!(p.name, "position");
        assert_eq!(p.value, "(1, 2, 3)");
    }

    #[test]
    fn property_sheet_empty() {
        let sheet = PropertySheet::new();
        assert!(sheet.is_empty());
        assert_eq!(sheet.len(), 0);
    }

    #[test]
    fn property_sheet_push_and_query() {
        let mut sheet = PropertySheet::new();
        sheet.push(Property::new("Transform", "position", "(0, 0, 0)"));
        sheet.push(Property::new("Transform", "rotation", "(0, 0, 0)"));
        sheet.push(Property::new("Material", "color", "red"));

        assert_eq!(sheet.len(), 3);
        assert_eq!(sheet.by_category("Transform").len(), 2);
        assert_eq!(sheet.by_category("Material").len(), 1);
        assert_eq!(sheet.by_category("Audio").len(), 0);
    }

    #[test]
    fn categories_ordered() {
        let mut sheet = PropertySheet::new();
        sheet.push(Property::new("B", "x", "1"));
        sheet.push(Property::new("A", "y", "2"));
        sheet.push(Property::new("B", "z", "3"));

        let cats = sheet.categories();
        assert_eq!(cats, vec!["B", "A"]);
    }

    #[test]
    fn property_equality() {
        let a = Property::new("T", "x", "1");
        let b = Property::new("T", "x", "1");
        assert_eq!(a, b);
    }

    #[test]
    fn property_serde_serialize() {
        let p = Property::new("Transform", "pos", "(1,2,3)");
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("Transform"));
        assert!(json.contains("pos"));
        assert!(json.contains("(1,2,3)"));
    }
}
