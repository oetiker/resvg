// Copyright 2026 the Resvg Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

// How a strike is spaced is not something an SVG can ask for, so these render
// their own documents and measure the result rather than compare against a
// reference image.

use crate::GLOBAL_FONTDB;

/// Renders `svg` at 1:1, so that a user-space unit is a device pixel.
fn render(svg: &str) -> tiny_skia::Pixmap {
    let opt = usvg::Options {
        fontdb: GLOBAL_FONTDB.clone(),
        ..usvg::Options::default()
    };
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

/// The rightmost inked column of `text` set at `font_size`, or `None` when
/// nothing was drawn.
fn last_inked_column(text: &str, font_size: u32) -> Option<u32> {
    let svg = format!(
        r#"<svg viewBox="0 0 300 60" width="300" height="60" xmlns="http://www.w3.org/2000/svg">
            <text x="20" y="40" font-family="Bitmap Mono" font-size="{font_size}">{text}</text>
        </svg>"#
    );
    let pixmap = render(&svg);
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
    let one = last_inked_column("M", font_size).expect("nothing was drawn");
    let many = last_inked_column(&"M".repeat(n), font_size).expect("nothing was drawn");
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
