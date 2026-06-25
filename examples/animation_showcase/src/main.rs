use ferrite::*;

fn main() {
    let checked = use_state(false);
    
    // We can also animate state directly and bind it to visual elements!
    let expanded = use_state(false);
    
    // Bouncy spring simulation for size
    let size_anim = use_spring(move || if expanded.get() { 1.0 } else { 0.0 }, SpringConfig::bouncy());
    
    // Elastic tween animation for color
    let color_anim = use_tween(move || if expanded.get() { 1.0 } else { 0.0 }, 0.6, |t| {
        let c4 = (2.0 * std::f32::consts::PI) / 3.0;
        if t == 0.0 { 0.0 }
        else if t == 1.0 { 1.0 }
        else { 2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0 }
    });

    let view = col([
        text("Ferrite Animations").size(32.0),
        spacer().height(20.0),
        
        // Fluid Checkbox (internally uses use_spring for scaling/fading)
        checkbox("Fluid Checkbox Example", checked),
        spacer().height(30.0),
        
        divider().width(400.0),
        spacer().height(30.0),
        
        button("Click me to Trigger Animations!", move || expanded.set(!expanded.get())),
        spacer().height(30.0),
        
        text("Spring Physics (Bouncy):"),
        // We use a slider as a visual gauge to show the spring bouncing back and forth
        slider(size_anim, 0.0, 1.0).width(300.0),
        spacer().height(20.0),
        
        text("Tween Animation (Elastic Ease-Out):"),
        // We use a slider as a visual gauge to show the tween elasticity
        slider(color_anim, 0.0, 1.0).width(300.0),
    ])
    .padding(40.0)
    .gap(10.0);

    ferrite::run("Animation Showcase", (800, 600), view);
}
