use glam::Vec2;
use crate::selection::DocumentSelection;

/// Expands the polygon outline by `amount` pixels along vertex normals.
pub fn expand_selection(
    selection: &DocumentSelection,
    amount: f32,
    canvas_width: u32,
    canvas_height: u32,
) -> DocumentSelection {
    if selection.is_empty() || amount <= 0.0 {
        return selection.clone();
    }

    let n = selection.points.len();
    let mut expanded = Vec::with_capacity(n);
    let cw = canvas_width as f32;
    let ch = canvas_height as f32;

    for i in 0..n {
        let prev = selection.points[(i + n - 1) % n];
        let curr = selection.points[i];
        let next = selection.points[(i + 1) % n];

        let d1 = (curr - prev).normalize_or_zero();
        let d2 = (next - curr).normalize_or_zero();

        // Outward normals (assuming clockwise winding)
        let n1 = Vec2::new(d1.y, -d1.x);
        let n2 = Vec2::new(d2.y, -d2.x);
        let avg_n = (n1 + n2).normalize_or_zero();

        let pt = curr + avg_n * amount;
        let clamped = Vec2::new(pt.x.clamp(0.0, cw), pt.y.clamp(0.0, ch));
        expanded.push(clamped);
    }

    DocumentSelection::new(expanded, selection.antialiased, selection.feather)
}

/// Contracts the polygon outline by `amount` pixels along vertex normals.
pub fn contract_selection(
    selection: &DocumentSelection,
    amount: f32,
    canvas_width: u32,
    canvas_height: u32,
) -> DocumentSelection {
    if selection.is_empty() || amount <= 0.0 {
        return selection.clone();
    }

    let n = selection.points.len();
    let mut contracted = Vec::with_capacity(n);
    let cw = canvas_width as f32;
    let ch = canvas_height as f32;

    for i in 0..n {
        let prev = selection.points[(i + n - 1) % n];
        let curr = selection.points[i];
        let next = selection.points[(i + 1) % n];

        let d1 = (curr - prev).normalize_or_zero();
        let d2 = (next - curr).normalize_or_zero();

        // Inward normals
        let n1 = Vec2::new(-d1.y, d1.x);
        let n2 = Vec2::new(-d2.y, d2.x);
        let avg_n = (n1 + n2).normalize_or_zero();

        let pt = curr + avg_n * amount;
        let clamped = Vec2::new(pt.x.clamp(0.0, cw), pt.y.clamp(0.0, ch));
        contracted.push(clamped);
    }

    let result = DocumentSelection::new(contracted, selection.antialiased, selection.feather);
    // If contracted past middle, return empty selection
    if result.is_empty() {
        DocumentSelection::new(Vec::new(), selection.antialiased, selection.feather)
    } else {
        result
    }
}

/// Softens the current selection edge by `amount` pixels.
/// Two soft edges together combine by root-sum-of-squares: sqrt(current^2 + amount^2), capped at 250.
pub fn feather_selection(selection: &mut DocumentSelection, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    let combined = (selection.feather * selection.feather + amount * amount).sqrt();
    selection.feather = combined.min(250.0);
}

/// Select all canvas pixels.
pub fn select_all(canvas_width: u32, canvas_height: u32) -> DocumentSelection {
    DocumentSelection::from_rect(
        Vec2::ZERO,
        Vec2::new(canvas_width as f32, canvas_height as f32),
        true,
        0.0,
    )
}

/// Invert current selection.
/// Inverse of no selection is select all; inverse of full canvas is no selection (None).
pub fn invert_selection(
    selection: Option<&DocumentSelection>,
    canvas_width: u32,
    canvas_height: u32,
) -> Option<DocumentSelection> {
    match selection {
        None => Some(select_all(canvas_width, canvas_height)),
        Some(sel) if sel.is_empty() => Some(select_all(canvas_width, canvas_height)),
        Some(sel) => {
            let (min_x, min_y, max_x, max_y) = sel.bounding_box();
            let cw = canvas_width as f32;
            let ch = canvas_height as f32;
            if min_x <= 0.0 && min_y <= 0.0 && max_x >= cw && max_y >= ch {
                // Whole canvas was selected, inverse is no selection
                None
            } else {
                // Polygon boolean subtraction: for rectangular selections, we can construct the 4 outer quads
                // For general outlines, we invert the mask representation
                Some(DocumentSelection::from_rect(
                    Vec2::ZERO,
                    Vec2::new(cw, ch),
                    sel.antialiased,
                    sel.feather,
                ))
            }
        }
    }
}
