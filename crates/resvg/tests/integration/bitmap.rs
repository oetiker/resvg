// Copyright 2026 the Resvg Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Whether a font's bitmap strikes are used is a host decision rather than
// something an SVG can ask for, so these render the same file the
// auto-generated `text_bitmap_font_monochrome` test uses.

use crate::render_with_bitmap_selector;

/// True when the face belongs to the named family.
fn is_family(db: &usvg::fontdb::Database, id: usvg::fontdb::ID, family: &str) -> bool {
    db.face(id)
        .map(|face| face.families.iter().any(|(name, _)| name == family))
        .unwrap_or(false)
}

#[test]
fn strikes_can_be_declined_for_a_font() {
    // `Bitmap Mono` carries a 16px and a 24px strike, which the file uses at
    // exactly those sizes. Declining them sends every size to the outline.
    assert_eq!(
        render_with_bitmap_selector(
            "tests/text/bitmap-font/monochrome",
            "tests/text/bitmap-font/monochrome-no-strikes",
            Box::new(|id, _, db| !is_family(db, id, "Bitmap Mono")),
        ),
        0
    );
}

#[test]
fn declining_strikes_for_another_family_changes_nothing() {
    // The control: the selector says no to a family the file never mentions,
    // so the rendering has to be the ordinary one.
    assert_eq!(
        render_with_bitmap_selector(
            "tests/text/bitmap-font/monochrome",
            "tests/text/bitmap-font/monochrome",
            Box::new(|id, _, db| !is_family(db, id, "Some Other Family")),
        ),
        0
    );
}
