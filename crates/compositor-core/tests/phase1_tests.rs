use glam::Vec2;
use compositor_core::{
    blend::LayerBlendMode,
    contract_selection, expand_selection, feather_selection, invert_selection, select_all,
    CanvasDocument, DocumentHistory, DocumentSelection,
    ImageLayer, LayerHierarchy,
};

#[test]
fn test_blend_mode_formulas() {
    let mode_multiply = LayerBlendMode::Multiply;
    assert!((mode_multiply.blend_channel(0.5, 0.5) - 0.25).abs() < 1e-5);

    let mode_screen = LayerBlendMode::Screen;
    assert!((mode_screen.blend_channel(0.5, 0.5) - 0.75).abs() < 1e-5);

    let mode_darken = LayerBlendMode::Darken;
    assert!((mode_darken.blend_channel(0.3, 0.7) - 0.3).abs() < 1e-5);

    let mode_lighten = LayerBlendMode::Lighten;
    assert!((mode_lighten.blend_channel(0.3, 0.7) - 0.7).abs() < 1e-5);

    let mode_diff = LayerBlendMode::Difference;
    assert!((mode_diff.blend_channel(0.8, 0.3) - 0.5).abs() < 1e-5);
}

#[test]
fn test_document_history_undo_redo() {
    let mut history = DocumentHistory::new(10, 1024 * 1024);
    let mut doc = CanvasDocument::new(800, 600);

    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert!(!history.is_modified());

    // Transaction 1: Add a layer
    history.begin("Add Layer", Some(&doc), None);
    let layer1 = ImageLayer::new_empty("Layer 1", Vec2::new(400.0, 300.0));
    let layer1_id = layer1.id;
    doc.layers.push(layer1);
    history.end(Some(&doc), Some(layer1_id));

    assert!(history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.undo_name(), "Add Layer");
    assert!(history.is_modified());

    // Undo
    let snap_before = history.undo().expect("Undo should succeed");
    assert_eq!(snap_before.document.unwrap().layers.len(), 0);
    assert!(!history.can_undo());
    assert!(history.can_redo());
    assert_eq!(history.redo_name(), "Add Layer");

    // Redo
    let snap_after = history.redo().expect("Redo should succeed");
    assert_eq!(snap_after.document.unwrap().layers.len(), 1);
    assert!(history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_history_nested_transactions_and_no_ops() {
    let mut history = DocumentHistory::new(10, 1024 * 1024);
    let doc = CanvasDocument::new(500, 500);

    // No-op edit should not add history entry
    history.begin("No-op", Some(&doc), None);
    history.end(Some(&doc), None);
    assert!(!history.can_undo());

    // Nested transactions
    history.begin("Outer", Some(&doc), None);
    history.begin("Inner", Some(&doc), None);
    let mut doc2 = doc.clone();
    doc2.layers.push(ImageLayer::new_empty("L1", Vec2::new(10.0, 10.0)));
    history.end(Some(&doc2), None); // Inner end: depth now 1, not committed yet
    assert_eq!(history.undo_count(), 0);

    history.end(Some(&doc2), None); // Outer end: depth 0, committed!
    assert_eq!(history.undo_count(), 1);
    assert_eq!(history.undo_name(), "Outer");
}

#[test]
fn test_selection_geometry_and_rasterization() {
    let sel = DocumentSelection::from_rect(
        Vec2::new(10.0, 10.0),
        Vec2::new(30.0, 30.0),
        true,
        0.0,
    );

    assert!(!sel.is_empty());
    assert!(sel.contains_point(Vec2::new(20.0, 20.0)));
    assert!(!sel.contains_point(Vec2::new(5.0, 5.0)));
    assert!(!sel.contains_point(Vec2::new(50.0, 50.0)));

    let mask = sel.rasterize_mask(100, 100);
    assert_eq!(mask.len(), 10000);
    // Center point (20, 20) must be 255
    assert_eq!(mask[20 * 100 + 20], 255);
    // Outside point (5, 5) must be 0
    assert_eq!(mask[5 * 100 + 5], 0);
}

#[test]
fn test_selection_edits_expand_contract_feather() {
    let sel = DocumentSelection::from_rect(
        Vec2::new(20.0, 20.0),
        Vec2::new(80.0, 80.0),
        true,
        0.0,
    );

    let expanded = expand_selection(&sel, 10.0, 200, 200);
    let (ex_min_x, _ex_min_y, ex_max_x, _ex_max_y) = expanded.bounding_box();
    assert!(ex_min_x <= 20.0);
    assert!(ex_max_x >= 80.0);

    let contracted = contract_selection(&sel, 5.0, 200, 200);
    let (ct_min_x, _ct_min_y, ct_max_x, _ct_max_y) = contracted.bounding_box();
    assert!(ct_min_x >= 20.0);
    assert!(ct_max_x <= 80.0);

    let mut feathered = sel.clone();
    feather_selection(&mut feathered, 4.0);
    assert!((feathered.feather - 4.0).abs() < 1e-5);
    feather_selection(&mut feathered, 3.0);
    // sqrt(4^2 + 3^2) = 5
    assert!((feathered.feather - 5.0).abs() < 1e-4);
}

#[test]
fn test_selection_invert_and_select_all() {
    let all = select_all(1920, 1080);
    assert!(!all.is_empty());
    let bounds = all.bounding_box();
    assert_eq!(bounds, (0.0, 0.0, 1920.0, 1080.0));

    // Invert of None gives select all
    let inv_none = invert_selection(None, 800, 600);
    assert!(inv_none.is_some());
    assert_eq!(inv_none.unwrap().bounding_box(), (0.0, 0.0, 800.0, 600.0));

    // Invert of full canvas gives None
    let inv_all = invert_selection(Some(&all), 1920, 1080);
    assert!(inv_all.is_none());
}

#[test]
fn test_layer_hierarchy_tree_visibility_and_opacity() {
    let mut doc = CanvasDocument::new(1000, 1000);

    let mut group = ImageLayer::new_group("Folder");
    group.opacity = 0.5;
    let group_id = group.id;
    doc.layers.push(group);

    let mut child = ImageLayer::new_empty("Child Layer", Vec2::new(100.0, 100.0));
    child.parent_id = Some(group_id);
    child.opacity = 0.5;
    let child_id = child.id;
    doc.layers.push(child);

    // Validate hierarchy passes
    assert!(LayerHierarchy::validate(&doc).is_ok());

    // Both are visible
    let visible_ids = LayerHierarchy::effective_visible_ids(&doc);
    assert!(visible_ids.contains(&group_id));
    assert!(visible_ids.contains(&child_id));

    // Effective opacity: 0.5 * 0.5 = 0.25
    let eff_op = LayerHierarchy::effective_opacity(&doc, child_id);
    assert!((eff_op - 0.25).abs() < 1e-5);

    // Hide parent group -> child should become effectively hidden
    doc.layer_mut(group_id).unwrap().is_visible = false;
    let visible_ids2 = LayerHierarchy::effective_visible_ids(&doc);
    assert!(!visible_ids2.contains(&child_id));
}

#[test]
fn test_layer_hierarchy_cycle_rejection() {
    let mut doc = CanvasDocument::new(500, 500);
    let mut g1 = ImageLayer::new_group("G1");
    let mut g2 = ImageLayer::new_group("G2");
    let id1 = g1.id;
    let id2 = g2.id;
    g1.parent_id = Some(id2);
    g2.parent_id = Some(id1); // Cycle!
    doc.layers.push(g1);
    doc.layers.push(g2);

    assert!(LayerHierarchy::validate(&doc).is_err());
}
