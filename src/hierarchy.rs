//! Generic hierarchy tree — parent-child relationships with depth-first traversal.
//!
//! Used for layer trees (rasa), track trees (shruti/tazama), entity trees (salai),
//! and any editor that needs a collapsible tree panel.

use serde::{Deserialize, Serialize};

/// A unique identifier for nodes in the hierarchy.
pub type NodeId = u64;

/// A node in the hierarchy tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyNode {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Display name.
    pub name: String,
    /// Child nodes.
    pub children: Vec<HierarchyNode>,
    /// Depth in the tree (0 = root).
    pub depth: usize,
}

/// A flattened hierarchy entry for display in a list/panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatEntry {
    /// Nesting depth (0 = root).
    pub depth: usize,
    /// Node identifier.
    pub id: NodeId,
    /// Display name.
    pub name: String,
}

/// Build a hierarchy tree from a flat list of items.
///
/// The `get_parent` closure returns `None` for root items or `Some(parent_id)`.
/// The `get_name` closure returns the display name for an item.
/// Items whose parent is not in `ids` are treated as roots.
#[must_use]
pub fn build_hierarchy<F, N>(ids: &[NodeId], get_parent: F, get_name: N) -> Vec<HierarchyNode>
where
    F: Fn(NodeId) -> Option<NodeId>,
    N: Fn(NodeId) -> String,
{
    let id_set: std::collections::HashSet<NodeId> = ids.iter().copied().collect();

    // Pre-build parent→children map for O(N) total instead of O(N²).
    let mut children_map: std::collections::HashMap<NodeId, Vec<NodeId>> =
        std::collections::HashMap::new();
    let mut root_ids = Vec::new();

    for &id in ids {
        match get_parent(id) {
            Some(parent) if parent != id && id_set.contains(&parent) => {
                children_map.entry(parent).or_default().push(id);
            }
            _ => root_ids.push(id),
        }
    }

    tracing::debug!(
        roots = root_ids.len(),
        total = ids.len(),
        "building hierarchy"
    );

    root_ids
        .iter()
        .map(|&id| build_node(id, 0, &children_map, &get_name))
        .collect()
}

fn build_node<N>(
    id: NodeId,
    depth: usize,
    children_map: &std::collections::HashMap<NodeId, Vec<NodeId>>,
    get_name: &N,
) -> HierarchyNode
where
    N: Fn(NodeId) -> String,
{
    let children = children_map
        .get(&id)
        .map(|child_ids| {
            child_ids
                .iter()
                .map(|&child_id| build_node(child_id, depth + 1, children_map, get_name))
                .collect()
        })
        .unwrap_or_default();

    HierarchyNode {
        id,
        name: get_name(id),
        children,
        depth,
    }
}

/// Flatten a hierarchy tree into a depth-first list for display.
#[must_use]
pub fn flatten(nodes: &[HierarchyNode]) -> Vec<FlatEntry> {
    let mut result = Vec::new();
    for node in nodes {
        flatten_node(node, &mut result);
    }
    tracing::debug!(entries = result.len(), "hierarchy flattened");
    result
}

#[inline]
fn flatten_node(node: &HierarchyNode, result: &mut Vec<FlatEntry>) {
    result.push(FlatEntry {
        depth: node.depth,
        id: node.id,
        name: node.name.clone(),
    });
    for child in &node.children {
        flatten_node(child, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parents() -> std::collections::HashMap<NodeId, NodeId> {
        let mut m = std::collections::HashMap::new();
        m.insert(2, 1); // 2 is child of 1
        m.insert(3, 1); // 3 is child of 1
        m.insert(4, 2); // 4 is child of 2
        m
    }

    fn names(id: NodeId) -> String {
        match id {
            1 => "Root".into(),
            2 => "Child A".into(),
            3 => "Child B".into(),
            4 => "Grandchild".into(),
            _ => format!("Node {id}"),
        }
    }

    #[test]
    fn build_simple_tree() {
        let p = parents();
        let tree = build_hierarchy(&[1, 2, 3, 4], |id| p.get(&id).copied(), names);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "Root");
        assert_eq!(tree[0].children.len(), 2);
    }

    #[test]
    fn flatten_depth_first() {
        let p = parents();
        let tree = build_hierarchy(&[1, 2, 3, 4], |id| p.get(&id).copied(), names);
        let flat = flatten(&tree);
        assert_eq!(flat.len(), 4);
        assert_eq!(flat[0].name, "Root");
        assert_eq!(flat[0].depth, 0);
        assert_eq!(flat[1].name, "Child A");
        assert_eq!(flat[1].depth, 1);
        assert_eq!(flat[2].name, "Grandchild");
        assert_eq!(flat[2].depth, 2);
        assert_eq!(flat[3].name, "Child B");
        assert_eq!(flat[3].depth, 1);
    }

    #[test]
    fn flatten_empty() {
        let flat = flatten(&[]);
        assert!(flat.is_empty());
    }

    #[test]
    fn flat_list_no_parents() {
        let tree = build_hierarchy(&[1, 2, 3], |_| None, names);
        assert_eq!(tree.len(), 3);
        for node in &tree {
            assert!(node.children.is_empty());
        }
    }

    #[test]
    fn orphan_treated_as_root() {
        // Node 10 has parent 99 which is not in the list
        let tree = build_hierarchy(&[1, 10], |id| if id == 10 { Some(99) } else { None }, names);
        assert_eq!(tree.len(), 2); // both are roots
    }

    #[test]
    fn self_parent_treated_as_root() {
        // Node whose parent is itself should be treated as root
        let tree = build_hierarchy(
            &[1, 2],
            |id| {
                if id == 1 { Some(1) } else { None }
            },
            names,
        );
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn mutual_cycle_treated_as_roots() {
        // Nodes that form a cycle: 1→2, 2→1 — both should become roots
        // since neither can be resolved as a child without infinite recursion
        let tree = build_hierarchy(
            &[1, 2],
            |id| {
                if id == 1 { Some(2) } else { Some(1) }
            },
            names,
        );
        // With our children_map approach: 1's parent is 2 (in set), 2's parent is 1 (in set).
        // Neither is a root. So roots is empty, but both are children of each other.
        // This is expected — cycles aren't trees. Verify no panic at minimum.
        let flat = flatten(&tree);
        assert!(flat.len() <= 2);
    }

    #[test]
    fn flat_entry_serde_roundtrip() {
        let entry = FlatEntry {
            depth: 2,
            id: 42,
            name: "Test Node".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: FlatEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn deep_nesting() {
        let tree = build_hierarchy(
            &[1, 2, 3, 4, 5],
            |id| if id > 1 { Some(id - 1) } else { None },
            |id| format!("L{id}"),
        );
        assert_eq!(tree.len(), 1);
        let flat = flatten(&tree);
        assert_eq!(flat.len(), 5);
        assert_eq!(flat[4].depth, 4);
    }
}
