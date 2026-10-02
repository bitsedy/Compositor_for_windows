use std::collections::{HashMap, HashSet};
use crate::layer::{CanvasDocument, ImageLayer, LayerId};

/// Hierarchy entry with tree depth and effective visibility.
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchyEntry<'a> {
    pub layer: &'a ImageLayer,
    pub depth: usize,
    pub effective_visible: bool,
}

pub struct LayerHierarchy;

impl LayerHierarchy {
    /// Generates flattened hierarchy entries visiting parent-child trees up to depth 64.
    pub fn entries<'a>(
        document: &'a CanvasDocument,
        top_first: bool,
        collapsed: &HashSet<LayerId>,
    ) -> Vec<HierarchyEntry<'a>> {
        let mut children_map: HashMap<Option<LayerId>, Vec<&'a ImageLayer>> = HashMap::new();
        for layer in &document.layers {
            children_map.entry(layer.parent_id).or_default().push(layer);
        }

        let mut result = Vec::new();
        Self::visit(
            None,
            0,
            true,
            &children_map,
            top_first,
            collapsed,
            &mut result,
        );
        result
    }

    fn visit<'a>(
        parent: Option<LayerId>,
        depth: usize,
        parent_visible: bool,
        children_map: &HashMap<Option<LayerId>, Vec<&'a ImageLayer>>,
        top_first: bool,
        collapsed: &HashSet<LayerId>,
        result: &mut Vec<HierarchyEntry<'a>>,
    ) {
        if depth > 64 {
            return;
        }
        let siblings = match children_map.get(&parent) {
            Some(s) => s,
            None => return,
        };

        let iter: Box<dyn Iterator<Item = &&'a ImageLayer>> = if top_first {
            Box::new(siblings.iter().rev())
        } else {
            Box::new(siblings.iter())
        };

        for layer in iter {
            let effective_visible = parent_visible && layer.is_visible;
            result.push(HierarchyEntry {
                layer,
                depth,
                effective_visible,
            });

            if layer.is_group && !collapsed.contains(&layer.id) {
                Self::visit(
                    Some(layer.id),
                    depth + 1,
                    effective_visible,
                    children_map,
                    top_first,
                    collapsed,
                    result,
                );
            }
        }
    }

    /// Set of layer IDs that are effectively visible (both themselves and all ancestor groups are visible).
    pub fn effective_visible_ids(document: &CanvasDocument) -> HashSet<LayerId> {
        let empty_collapsed = HashSet::new();
        Self::entries(document, false, &empty_collapsed)
            .into_iter()
            .filter(|e| e.effective_visible)
            .map(|e| e.layer.id)
            .collect()
    }

    /// Effective rendering opacity for a layer, multiplied down through parent folder chains (max depth 64).
    pub fn effective_opacity(document: &CanvasDocument, layer_id: LayerId) -> f32 {
        let layer = match document.layer(layer_id) {
            Some(l) => l,
            None => return 0.0,
        };

        let mut opacity = layer.opacity;
        let mut curr_parent = layer.parent_id;
        let mut depth = 0;

        while let Some(parent_id) = curr_parent {
            if depth >= 64 {
                break;
            }
            if let Some(parent_layer) = document.layer(parent_id) {
                opacity *= parent_layer.opacity;
                curr_parent = parent_layer.parent_id;
            } else {
                break;
            }
            depth += 1;
        }

        opacity.clamp(0.0, 1.0)
    }

    /// Validates hierarchy invariants:
    /// - No duplicate IDs
    /// - No cycles in parent links
    /// - Maximum nesting depth <= 64
    /// - Groups must not have raster files
    pub fn validate(document: &CanvasDocument) -> Result<(), &'static str> {
        let mut ids = HashSet::new();
        for layer in &document.layers {
            if !ids.insert(layer.id) {
                return Err("Duplicate layer ID found in hierarchy");
            }
            if layer.is_group && layer.asset_filename.is_some() {
                return Err("Group layer cannot have associated raster image file");
            }
        }

        for layer in &document.layers {
            let mut visited = HashSet::new();
            visited.insert(layer.id);
            let mut curr = layer.parent_id;
            while let Some(pid) = curr {
                if visited.len() > 64 {
                    return Err("Layer nesting depth exceeds maximum limit of 64");
                }
                if !visited.insert(pid) {
                    return Err("Cycle detected in layer parent hierarchy");
                }
                match document.layer(pid) {
                    Some(p) => {
                        if !p.is_group {
                            return Err("Layer parent must be a group layer");
                        }
                        curr = p.parent_id;
                    }
                    None => return Err("Layer references non-existent parent group"),
                }
            }
        }

        Ok(())
    }

    /// Gets all layers clipped to a base layer (where mask_source_id == base_layer_id).
    pub fn clipping_chain<'a>(
        document: &'a CanvasDocument,
        base_layer_id: LayerId,
    ) -> Vec<&'a ImageLayer> {
        let mut chain = Vec::new();
        if let Some(base) = document.layer(base_layer_id) {
            chain.push(base);
            for layer in &document.layers {
                if layer.mask_source_id == Some(base_layer_id) {
                    chain.push(layer);
                }
            }
        }
        chain
    }
}
