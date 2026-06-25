use ferrite::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
enum Tab {
    Basic,
    Advanced,
}

#[derive(Clone)]
struct Theme {
    accent: Color,
}

fn main() {
    let active_tab = use_state(Tab::Basic);
    let theme = use_state(Theme { accent: Color::rgb(0.21, 0.43, 0.86) });
    provide(theme);

    ferrite::run(
        "Ferrite Showcase",
        (540, 480),
        col([
            text("Ferrite UI")
                .size(28.0)
                .color(Color::rgb(0.1, 0.1, 0.15)),
            
            row([
                button("Basic", move || active_tab.set(Tab::Basic))
                    .background(Color::rgb(0.9, 0.9, 0.92))
                    .foreground(Color::rgb(0.3, 0.3, 0.35)),
                button("Advanced", move || active_tab.set(Tab::Advanced))
                    .background(Color::rgb(0.9, 0.9, 0.92))
                    .foreground(Color::rgb(0.3, 0.3, 0.35)),
            ]).gap(12.0),

            divider(),

            switch(active_tab, [
                (Tab::Basic, basic_view()),
                (Tab::Advanced, advanced_view()),
            ]),
        ])
        .padding(32.0)
        .gap(20.0),
    );
}

fn basic_view() -> AnyView {
    let count = use_state(0i32);
    let name = use_state(String::new());

    col([
        text("Basic Controls").size(20.0).color(Color::rgb(0.3, 0.3, 0.35)),
        
        row([
            label(move || format!("Count: {}", count.get())).size(18.0),
            spacer(),
            button("+", move || count.update(|c| *c += 1)),
        ]).align(AlignItems::Center).width(300.0),

        row([
            text("Name:").size(18.0),
            spacer(),
            input(name, "Type something...").width(200.0),
        ]).align(AlignItems::Center).width(300.0),
    ]).gap(16.0)
}

fn advanced_view() -> AnyView {
    let volume = use_state(50.0);
    let enable_sound = use_state(true);

    let theme: Signal<Theme> = inject();

    col([
        text("Advanced Settings").size(20.0).color(Color::rgb(0.3, 0.3, 0.35)),
        
        col([
            row([
                text("Master Volume").size(18.0),
                spacer(),
                slider(volume, 0.0, 100.0),
            ]).align(AlignItems::Center),

            row([
                text("Enable Sound").size(18.0),
                spacer(),
                checkbox("", enable_sound),
            ]).align(AlignItems::Center),
        ])
        .width(400.0)
        .padding(16.0)
        .corner_radius(8.0)
        .background(Color::rgb(0.94, 0.95, 0.97))
        .gap(12.0),

        // Show label only when sound is enabled
        label(move || format!("Volume is at {:.0}%", volume.get()))
            .color(theme.get().accent)
            .visible_when(move || enable_sound.get()),

        spacer(),

        button("Toggle Theme Color", move || {
            theme.update(|t| {
                if t.accent == Color::rgb(0.21, 0.43, 0.86) {
                    t.accent = Color::rgb(0.86, 0.21, 0.43); // Red
                } else {
                    t.accent = Color::rgb(0.21, 0.43, 0.86); // Blue
                }
            })
        }).background(Color::rgb(0.1, 0.1, 0.15)),
    ]).gap(16.0)
}
