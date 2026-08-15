// Copyright 2026 the Resvg Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Bitmap strikes: whether they are used at all is a host decision, and how
// they are spaced and placed is not something an SVG can ask for. So these
// either render against a reference image or measure their own documents.

use crate::{GLOBAL_FONTDB, render_with_bitmap_selector};

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

/// Renders `svg` at 1:1, so that a user-space unit is a device pixel.
fn render(svg: &str) -> tiny_skia::Pixmap {
    render_with(svg, true)
}

/// Renders `svg` at 1:1 with a resolver that allows or declines every font's
/// bitmap strikes.
fn render_with(svg: &str, allow_strikes: bool) -> tiny_skia::Pixmap {
    let mut opt = usvg::Options {
        fontdb: GLOBAL_FONTDB.clone(),
        ..usvg::Options::default()
    };
    opt.font_resolver.select_bitmap = Box::new(move |_, _, _| allow_strikes);
    let tree = usvg::Tree::from_data(svg.as_bytes(), &opt).unwrap();
    let size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    pixmap
}

/// Every distinct alpha value in the rendering.
fn alphas(svg: &str) -> std::collections::BTreeSet<u8> {
    render(svg).pixels().iter().map(|p| p.alpha()).collect()
}

/// The text is placed at a whole and at a fractional x, in a document that is
/// rendered unscaled, so a user-space unit is a device pixel.
fn text_at(x: &str) -> String {
    format!(
        r#"<svg viewBox="0 0 300 200" width="300" height="200" xmlns="http://www.w3.org/2000/svg">
            <text x="{x}" y="40" font-family="Bitmap Mono" font-size="16">Bitmap 16</text>
        </svg>"#
    )
}

#[test]
fn a_strike_is_blitted_onto_whole_pixels() {
    // A strike is a picture of one exact pixel grid. Painting it over a
    // rectangle with fractional edges anti-aliases its outermost row and
    // column, which on a monochrome target is the difference between a stem
    // and no stem. Spacing a glyph by its strike's advance keeps a run on the
    // grid on its own, but only as long as the document puts the run there —
    // a fractional x, or a fractional letter-spacing, moves it off again.
    for x in ["20", "20.25", "20.5", "20.75"] {
        assert_eq!(
            alphas(&text_at(x)),
            [0, 255].into_iter().collect(),
            "a strike placed at x={x} was not blitted onto whole pixels"
        );
    }
}

/// The rightmost inked column of `text` set at `font_size`, or `None` when
/// nothing was drawn.
fn last_inked_column(text: &str, font_size: u32) -> Option<u32> {
    last_inked_column_with(text, font_size, true)
}

/// As [`last_inked_column`], with the host allowing or declining strikes.
fn last_inked_column_with(text: &str, font_size: u32, allow_strikes: bool) -> Option<u32> {
    let svg = format!(
        r#"<svg viewBox="0 0 300 60" width="300" height="60" xmlns="http://www.w3.org/2000/svg">
            <text x="20" y="40" font-family="Bitmap Mono" font-size="{font_size}">{text}</text>
        </svg>"#
    );
    let pixmap = render_with(&svg, allow_strikes);
    let width = pixmap.width();
    pixmap
        .pixels()
        .iter()
        .enumerate()
        .filter(|(_, p)| p.alpha() != 0)
        .map(|(i, _)| i as u32 % width)
        .max()
}

/// The advance the renderer actually used, in pixels: a run of `n` identical
/// glyphs reaches `(n - 1)` advances further right than a single one, whatever
/// the glyph's own ink and bearings are.
fn measured_advance(font_size: u32, n: usize) -> f32 {
    measured_advance_with(font_size, n, true)
}

/// As [`measured_advance`], with the host allowing or declining strikes.
fn measured_advance_with(font_size: u32, n: usize, allow_strikes: bool) -> f32 {
    let one = last_inked_column_with("M", font_size, allow_strikes).expect("nothing was drawn");
    let many = last_inked_column_with(&"M".repeat(n), font_size, allow_strikes)
        .expect("nothing was drawn");
    (many - one) as f32 / (n - 1) as f32
}

/// A strike carries its own advance, drawn in whole pixels for its own pixel
/// size, and it is not the outline's advance scaled.
///
/// `Bitmap Mono` comes from Terminus, whose cell is 8x14 at 14px and 8x16 at
/// 16px. A single `hmtx` advance can only scale, so at 14px the outline says
/// 7px where the strike says 8 — the strike is the one that is right, and the
/// two sizes cannot both be expressed by any one `hmtx` value.
#[test]
fn a_strike_is_spaced_by_its_own_advance() {
    // 14px has a strike, and it is the size where the outline disagrees.
    assert_eq!(
        measured_advance(14, 8),
        8.0,
        "a 14px strike was spaced by the outline's 7px advance"
    );

    // 16px has a strike too, and there the outline happens to agree. This is
    // the control: it holds either way, so on its own it proves nothing.
    assert_eq!(measured_advance(16, 8), 8.0);

    // 20px has no strike, so the outline is drawn and its advance is correct.
    assert_eq!(measured_advance(20, 8), 10.0);
}

/// Declining a font's strikes has to take its spacing with it.
///
/// A host that says no gets the outline drawn, so it must get the outline's
/// advance too. Leaving the strike's advance in place would space glyphs for
/// a picture that is never painted — at 14px, an 8px pitch around a 7px glyph.
#[test]
fn declining_strikes_also_declines_their_advance() {
    // 14px is the size where the two disagree, so it is the only one that can
    // tell which of them was used.
    assert_eq!(
        measured_advance_with(14, 8, true),
        8.0,
        "allowed strikes should be spaced by the strike"
    );
    assert_eq!(
        measured_advance_with(14, 8, false),
        7.0,
        "declined strikes should be spaced by the outline"
    );
}
