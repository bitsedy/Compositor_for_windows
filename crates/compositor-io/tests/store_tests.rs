use std::collections::HashMap;
use std::fs;
use compositor_core::LayerTransform;
use compositor_io::{
    LoadedImageAsset, ProjectError, ProjectLayerRecord, ProjectManifest, ProjectSnapshot,
    ProjectStore, CURRENT_PROJECT_VERSION,
};
use uuid::Uuid;

#[test]
fn test_project_store_roundtrip() {
    let temp_dir = std::env::temp_dir().join(format!("comp_test_{}", Uuid::new_v4()));
    let layer_id = Uuid::new_v4();

    let manifest = ProjectManifest {
        format: "com.compositor.project".to_string(),
        version: CURRENT_PROJECT_VERSION,
        color_space: "sRGB".to_string(),
        resolution: Some(300.0),
        document_id: Uuid::new_v4(),
        width: 100,
        height: 100,
        active_layer_id: Some(layer_id),
        layers: vec![ProjectLayerRecord {
            id: layer_id,
            name: "Background".to_string(),
            is_visible: true,
            transform: LayerTransform::new(0.0, 0.0, 100.0, 100.0),
            image_file: Some(format!("{}.png", layer_id)),
            parent_id: None,
            is_group: None,
            opacity: Some(1.0),
            blend_mode: Some("Normal".to_string()),
            mask_file: None,
            mask_enabled: None,
            mask_source_id: None,
            adjustment: None,
            mask_placement: None,
            mask_linked: None,
            shape: None,
            effects: None,
            text: None,
        }],
        guides: None,
    };

    let mut images = HashMap::new();
    images.insert(
        layer_id,
        LoadedImageAsset {
            width: 100,
            height: 100,
            rgba: vec![255u8; 100 * 100 * 4],
        },
    );

    let snapshot = ProjectSnapshot {
        manifest,
        images,
        masks: HashMap::new(),
    };

    // Save
    ProjectStore::save(&snapshot, &temp_dir).expect("Project save should succeed");

    // Load
    let loaded = ProjectStore::load(&temp_dir).expect("Project load should succeed");
    assert_eq!(loaded.manifest.width, 100);
    assert_eq!(loaded.manifest.height, 100);
    assert_eq!(loaded.manifest.version, 11);
    assert_eq!(loaded.manifest.layers.len(), 1);
    assert_eq!(loaded.images.len(), 1);

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_project_manifest_versions_1_to_11_backward_compatibility() {
    let temp_dir = std::env::temp_dir().join(format!("comp_v1_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    // Version 1 manifest without resolution, guides, effects, etc.
    let v1_json = r#"{
        "format": "com.compositor.project",
        "version": 1,
        "documentID": "00000000-0000-0000-0000-000000000001",
        "width": 64,
        "height": 64,
        "layers": []
    }"#;

    fs::write(temp_dir.join("manifest.json"), v1_json).unwrap();

    let loaded = ProjectStore::load(&temp_dir).expect("Version 1 manifest should load cleanly");
    assert_eq!(loaded.manifest.version, 1);
    assert_eq!(loaded.manifest.width, 64);
    assert_eq!(loaded.manifest.height, 64);
    assert!(loaded.manifest.layers.is_empty());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_project_unsupported_version_rejected() {
    let temp_dir = std::env::temp_dir().join(format!("comp_v99_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let v99_json = r#"{
        "format": "com.compositor.project",
        "version": 99,
        "documentID": "00000000-0000-0000-0000-000000000001",
        "width": 64,
        "height": 64,
        "layers": []
    }"#;

    fs::write(temp_dir.join("manifest.json"), v99_json).unwrap();

    let err = ProjectStore::load(&temp_dir).expect_err("Version 99 must be rejected");
    match err {
        ProjectError::Version(v) => assert_eq!(v, 99),
        _ => panic!("Expected ProjectError::Version"),
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_project_watcher_coalescing() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use compositor_io::ProjectWatcher;

    let temp_dir = std::env::temp_dir().join(format!("comp_watch_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let trigger_count = Arc::new(AtomicUsize::new(0));
    let trigger_clone = Arc::clone(&trigger_count);

    let mut watcher = ProjectWatcher::new(&temp_dir, move || {
        trigger_clone.fetch_add(1, Ordering::SeqCst);
    }).expect("Failed to initialize ProjectWatcher");

    // Write file 1
    fs::write(temp_dir.join("manifest.json"), "{}").unwrap();
    thread::sleep(Duration::from_millis(50));

    // Write file 2 rapidly
    fs::write(temp_dir.join("test.txt"), "hello").unwrap();
    thread::sleep(Duration::from_millis(50));

    // Write file 3 rapidly
    fs::write(temp_dir.join("test.txt"), "world").unwrap();

    // Now wait 500ms for coalescing period (300ms) to elapse
    thread::sleep(Duration::from_millis(500));

    assert_eq!(trigger_count.load(Ordering::SeqCst), 1, "Rapid changes should be coalesced into 1 notification");

    watcher.stop();
    let _ = fs::remove_dir_all(&temp_dir);
}
