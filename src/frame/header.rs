//! The header above every page: its tabs and the account chip.

use super::*;

impl Launcher {
    pub(super) fn header(&mut self, cx: &mut Context<Self>) -> Div {
        let p = palette();
        let active = self.ui.page.active_tab();
        // Below 900 px with the sidebar open; never with it closed (the window is 800 px at least).
        let compact = self.bar_width() < 750.;
        // Keep every destination readable; the account chip can shorten its label.
        let mut tabs = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(if self.window_width < 1000. { 2. } else { 10. }));
        for (dest, id, icon, label) in [
            (Page::Home, "nav-home", "home", tr("Home")),
            (Page::Discover, "nav-mods", "puzzle", tr("Mods")),
            (Page::Packs, "nav-packs", "package", tr("Packs")),
            (Page::Join, "nav-servers", "server", tr("Servers")),
            (Page::Skins, "nav-skins", "shirt", tr("Skins")),
        ] {
            let on = active == dest;
            let colour = if on { p.chrome_text } else { p.chrome_muted };
            tabs = tabs.child(
                div()
                    .id(id)
                    .reach()
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .gap(px(7.))
                    .px(px(if compact {
                        7.
                    } else if self.window_width < 1000. {
                        8.
                    } else {
                        14.
                    }))
                    .h(px(42.))
                    .whitespace_nowrap()
                    .text_color(rgb(colour))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(p.chrome_raised)))
                    .when(!compact, |d| {
                        d.child(blocks::icon(icon, 16., rgb(colour).into()))
                    })
                    .child(label)
                    .when(on, |d| {
                        d.child(
                            div()
                                .absolute()
                                .bottom_0()
                                .left(px(9.))
                                .right(px(9.))
                                .h(px(2.))
                                .bg(rgb(p.accent)),
                        )
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let to = if dest == Page::Join {
                            this.ui.last_server.unwrap_or(dest)
                        } else {
                            dest
                        };
                        this.navigate(to);
                        cx.notify();
                    })),
            );
        }
        let update = self.launcher_update_control(cx);
        let compact_account = self.window_width < 970. && update.is_some();
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .px(px(10.))
            .h(px(tokens::HEADER))
            .rounded(crate::tokens::r(crate::tokens::R_PANEL))
            .border_1()
            .border_color(p.rule())
            .bg(p.chrome_plate())
            .child(tabs)
            .child(
                div()
                    .flex()
                    .min_w_0()
                    .items_center()
                    .gap(px(10.))
                    .child(self.search_button(cx))
                    .children(update)
                    .child(self.account_chip(compact_account, cx)),
            )
    }
}
