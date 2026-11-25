//! The hosting page's small controls, labels and rows.
use super::*;

pub(super) fn title(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(tokens::DISPLAY))
        .font_weight(FontWeight::BOLD)
        .child(crate::selectable::selectable(text))
}

pub(super) fn label(text: &'static str) -> Div {
    caption(tr(text))
}

/// A small heading in capitals; `text` is already in the player's language.
pub(super) fn caption(text: &str) -> Div {
    div()
        .text_size(px(tokens::CAPTION))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(palette().muted))
        .child(crate::selectable::selectable(text.to_uppercase()))
}

pub(super) fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .rounded(crate::tokens::r(crate::tokens::R_CARD))
        .border_1()
        .border_color(palette().rule())
        .bg(palette().card())
}

/// The main action of the page, in cream.
pub(super) fn primary(b: Stateful<Div>) -> Stateful<Div> {
    b.bg(rgb(palette().text))
        .border_color(rgb(palette().text))
        .text_color(rgb(palette().background))
}

pub(super) fn dot(color: u32) -> Div {
    div()
        .flex_none()
        .size(px(8.))
        .rounded(crate::tokens::r(crate::tokens::R_SMALL))
        .bg(rgb(color))
}

/// A line that says something happened, with what can be done about it.
pub(super) fn notice(color: u32, text: impl Into<SharedString>, actions: Div) -> Div {
    card()
        .flex_row()
        .items_center()
        .gap(px(12.))
        .px(px(16.))
        .py(px(12.))
        .child(dot(color))
        .child(div().flex_1().min_w_0().child(text.into()))
        .child(actions.flex_none().flex().items_center().gap(px(6.)))
}

/// One choice among `options`, as chips. The group is one Tab stop with the ring around it, and
/// the arrow keys move the choice within it.
pub(super) fn chip_group(
    id: &'static str,
    options: &[(&'static str, &'static str)],
    current: &str,
    cx: &mut Context<Launcher>,
    set: impl Fn(&mut Launcher, &'static str) + Clone + 'static,
) -> Div {
    let chips = options
        .iter()
        .map(|&(value, name)| {
            let set = set.clone();
            let pick: Choose = Rc::new(move |this, _| set(this, value));
            (tr(name).into(), value == current, pick)
        })
        .collect();
    div().flex().child(choices(id, chips, cx).gap(px(6.)))
}

pub(super) fn phase_color(status: &Status) -> u32 {
    match status.phase.as_str() {
        "running" if status.crashes > 0 => palette().warning,
        "running" => palette().success,
        "stopped" => palette().line,
        _ => palette().warning,
    }
}

impl Launcher {
    /// A text field of the Servers page, drawn in this look.
    pub(super) fn host_field(
        &mut self,
        f: Field,
        placeholder: &'static str,
        width: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let focused = self.servers.has_focus(f);
        let invalid = self.servers.invalid.is_some_and(|(g, _)| g == f);
        self.field(f, placeholder, width, cx)
            .h(px(tokens::FIELD))
            .px(px(12.))
            .rounded(crate::tokens::r(crate::tokens::R_CONTROL))
            .border_1()
            .border_color(if invalid {
                rgb(palette().danger).into()
            } else if focused {
                rgb(palette().focus).into()
            } else {
                palette().input_edge()
            })
            .bg(rgb(palette().field))
            .text_size(px(tokens::LABEL))
            .text_color(rgb(palette().text))
    }

    pub(super) fn host_button(
        &self,
        id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        enabled: bool,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        let b = button(id, text);
        if enabled {
            b.on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                action(this, cx);
                cx.notify();
            }))
        } else {
            off(b)
        }
    }

    /// − value + for a number setting within `range`.
    pub(super) fn host_stepper(
        &self,
        key: &'static str,
        range: std::ops::RangeInclusive<i64>,
        cx: &mut Context<Self>,
    ) -> Div {
        let (section, field) = key.split_once('.').unwrap_or_default();
        let value: i64 = self.servers.shown(section, field).parse().unwrap_or(0);
        let step = |id: &'static str, sign: &'static str, to: i64, cx: &mut Context<Self>| {
            self.host_button(
                SharedString::from(format!("{key}-{id}")),
                sign,
                range.contains(&to),
                cx,
                move |this, _| this.set_setting(key, to.to_string()),
            )
            .w(px(32.))
            .px(px(0.))
        };
        div()
            .child(step("less", "−", value - 1, cx))
            .child(
                div()
                    .w(px(36.))
                    .flex()
                    .justify_center()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(value.to_string()),
            )
            .child(step("more", "+", value + 1, cx))
    }

    pub(super) fn host_choice(
        &self,
        key: &'static str,
        options: &[(&'static str, &'static str)],
        cx: &mut Context<Self>,
    ) -> Div {
        let (section, field) = key.split_once('.').unwrap_or_default();
        let current = self.servers.shown(section, field);
        chip_group(key, options, &current, cx, move |this, value| {
            this.set_setting(key, value.to_string())
        })
    }

    pub(super) fn host_person(name: String, about: Option<String>, face: Div, actions: Div) -> Div {
        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .child(face)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(name))
                    .children(about.map(hint)),
            )
            .child(actions.flex_none().flex().items_center().gap(px(6.)))
    }
}
