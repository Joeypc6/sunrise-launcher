use gpui::{div, prelude::*, px, Bounds, Context, Pixels, Window};

/// The frame's root as `frame::frame` lays it out: a block with the page flowing in it at the
/// window's size, then the resize edges.
struct Framed;

impl Render for Framed {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .child(div().size_full())
            .child(super::resize_edges())
    }
}

// GPUI Fast records `debug_selector` bounds only with debug assertions: the release
// workflow's `cargo test --release` has none to find, so it leaves this test out.
#[cfg(debug_assertions)]
#[test]
fn every_edge_of_the_window_takes_a_resize_press() {
    let mut app = gpui::TestAppContext::single();
    let (_, cx) = app.add_window_view(|_, _| Framed);
    cx.run_until_parked();
    let window = cx.update(|window, _| window.viewport_size());
    let edge = |cx: &mut gpui::VisualTestContext, id: &'static str| -> Bounds<Pixels> {
        cx.debug_bounds(id)
            .unwrap_or_else(|| panic!("{id} is drawn"))
    };
    let inside = |b: Bounds<Pixels>, id: &str| {
        assert!(
            b.origin.x >= px(0.)
                && b.origin.y >= px(0.)
                && b.bottom_right().x <= window.width
                && b.bottom_right().y <= window.height
                && b.size.width > px(0.)
                && b.size.height > px(0.),
            "{id} at {b:?} lies inside the {window:?} window"
        );
    };
    for id in [
        "resize-top",
        "resize-left",
        "resize-right",
        "resize-bottom",
        "resize-top-left",
        "resize-top-right",
        "resize-bottom-left",
        "resize-bottom-right",
    ] {
        inside(edge(cx, id), id);
    }
    assert_eq!(edge(cx, "resize-top").origin.y, px(0.));
    assert_eq!(edge(cx, "resize-left").origin.x, px(0.));
    assert_eq!(edge(cx, "resize-right").bottom_right().x, window.width);
    assert_eq!(edge(cx, "resize-bottom").bottom_right().y, window.height);
    assert_eq!(
        edge(cx, "resize-bottom-right").bottom_right(),
        gpui::point(window.width, window.height)
    );
}
