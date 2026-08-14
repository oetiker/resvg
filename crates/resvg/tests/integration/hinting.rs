// Copyright 2026 the Resvg Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Hinting is an `Options` setting rather than something an SVG can ask for, so
// these render the same file the auto-generated `text_hinting_sizes` test uses,
// once per configuration.

use crate::{render_hinted, render_with_hinting_resolver};

/// The configuration `sizes-mono.png` was rendered with.
fn mono() -> usvg::FontHintingOptions {
    usvg::FontHintingOptions {
        engine: usvg::FontHintingEngine::Auto,
        target: usvg::FontHintingTarget::Mono,
    }
}

/// True when the face belongs to the family the test file asks for.
fn is_family(db: &usvg::fontdb::Database, id: usvg::fontdb::ID, family: &str) -> bool {
    db.face(id)
        .map(|face| face.families.iter().any(|(name, _)| name == family))
        .unwrap_or(false)
}

#[test]
fn smooth() {
    let hinting = usvg::FontHintingOptions::default();
    assert_eq!(
        render_hinted("tests/text/hinting/sizes", "smooth", hinting),
        0
    );
}

#[test]
fn autohinter() {
    let hinting = usvg::FontHintingOptions {
        engine: usvg::FontHintingEngine::Auto,
        ..usvg::FontHintingOptions::default()
    };
    assert_eq!(
        render_hinted("tests/text/hinting/sizes", "auto", hinting),
        0
    );
}

#[test]
fn autohinter_mono() {
    // The TrueType interpreter barely distinguishes a mono target from a smooth
    // one, so this pairs it with the engine that does.
    let hinting = usvg::FontHintingOptions {
        engine: usvg::FontHintingEngine::Auto,
        target: usvg::FontHintingTarget::Mono,
    };
    assert_eq!(
        render_hinted("tests/text/hinting/sizes", "mono", hinting),
        0
    );
}

// The resolver lets a host pick hinting per font. These check it against the
// references the global setting already produces: turning hinting off for every
// font has to match the unhinted rendering, and turning it on has to match the
// globally hinted one.

#[test]
fn resolver_declining_every_font_matches_unhinted() {
    assert_eq!(
        render_with_hinting_resolver(
            "tests/text/hinting/sizes",
            "tests/text/hinting/sizes",
            Some(mono()),
            Box::new(|_, _, _, _| None),
        ),
        0
    );
}

#[test]
fn resolver_passing_the_global_through_matches_global_hinting() {
    assert_eq!(
        render_with_hinting_resolver(
            "tests/text/hinting/sizes",
            "tests/text/hinting/sizes-mono",
            Some(mono()),
            Box::new(|_, _, global, _| global),
        ),
        0
    );
}

// The pair below is the point of the feature: the same resolver, keyed on a
// different family, has to produce opposite results. The global is off in both,
// so any hinting can only have come from the resolver.

#[test]
fn resolver_hints_the_family_it_names() {
    assert_eq!(
        render_with_hinting_resolver(
            "tests/text/hinting/sizes",
            "tests/text/hinting/sizes-mono",
            None,
            Box::new(|id, _, _, db| is_family(db, id, "Noto Sans").then(mono)),
        ),
        0
    );
}

// A bitmap strike and a hinted outline in one document. Hinting grid-fits
// outlines, so it has nothing to say about a glyph that arrives as an image —
// these pin that down rather than leaving it to be discovered.

#[test]
fn hinting_changes_the_outline_font_in_a_mixed_document() {
    assert_eq!(
        render_hinted("tests/text/hinting/mixed-fonts", "mono", mono()),
        0
    );
}

#[test]
fn hinting_only_the_bitmap_font_leaves_the_document_unhinted() {
    // `Bitmap Mono` is drawn from its 16px strike here, so asking for hinting
    // on it has to be indistinguishable from asking for none at all.
    assert_eq!(
        render_with_hinting_resolver(
            "tests/text/hinting/mixed-fonts",
            "tests/text/hinting/mixed-fonts",
            None,
            Box::new(|id, _, _, db| is_family(db, id, "Bitmap Mono").then(mono)),
        ),
        0
    );
}

#[test]
fn declining_the_bitmap_font_matches_hinting_everything() {
    // The mirror image: hinting every font and hinting all but the bitmap one
    // have to agree, because the bitmap font was never affected either way.
    assert_eq!(
        render_with_hinting_resolver(
            "tests/text/hinting/mixed-fonts",
            "tests/text/hinting/mixed-fonts-mono",
            Some(mono()),
            Box::new(|id, _, global, db| (!is_family(db, id, "Bitmap Mono"))
                .then_some(global)
                .flatten()),
        ),
        0
    );
}

#[test]
fn resolver_leaves_families_it_does_not_name_unhinted() {
    assert_eq!(
        render_with_hinting_resolver(
            "tests/text/hinting/sizes",
            "tests/text/hinting/sizes",
            None,
            Box::new(|id, _, _, db| is_family(db, id, "Some Other Family").then(mono)),
        ),
        0
    );
}
