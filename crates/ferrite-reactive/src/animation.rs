use std::cell::RefCell;

pub type AnimCallback = Box<dyn FnMut(f32) -> bool>;

thread_local! {
    static ANIMATIONS: RefCell<Vec<AnimCallback>> = RefCell::new(Vec::new());
}

static mut WAKE_UP_FN: fn() = || {};

/// Set the global hook to wake up the UI event loop.
pub fn set_wake_up(f: fn()) {
    unsafe { WAKE_UP_FN = f; }
}

fn wake_up() {
    unsafe { WAKE_UP_FN(); }
}

/// Request an animation frame. The callback receives delta time (dt) in seconds.
/// Return `true` to keep the animation running, or `false` if it is finished.
pub fn request_animation_frame(cb: impl FnMut(f32) -> bool + 'static) {
    ANIMATIONS.with(|anims| anims.borrow_mut().push(Box::new(cb)));
    wake_up();
}

/// Ticks all active animations. Returns true if any animations are still running.
pub fn tick_animations(dt: f32) -> bool {
    let mut still_running = false;
    ANIMATIONS.with(|anims| {
        let mut fns = anims.replace(Vec::new());
        let mut keep = Vec::new();
        for mut cb in fns.drain(..) {
            if cb(dt) {
                keep.push(cb);
                still_running = true;
            }
        }
        anims.borrow_mut().extend(keep);
    });
    
    if still_running {
        wake_up();
    }
    
    still_running
}

pub mod easing {
    use std::f32::consts::PI;

    pub fn linear(t: f32) -> f32 { t }
    pub fn ease_in(t: f32) -> f32 { t * t }
    pub fn ease_out(t: f32) -> f32 { t * (2.0 - t) }
    pub fn ease_in_out(t: f32) -> f32 {
        if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
    }
    pub fn ease_out_back(t: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        let t = t - 1.0;
        1.0 + c3 * t.powi(3) + c1 * t.powi(2)
    }
    pub fn ease_out_elastic(t: f32) -> f32 {
        let c4 = (2.0 * PI) / 3.0;
        if t == 0.0 { 0.0 }
        else if t == 1.0 { 1.0 }
        else { 2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0 }
    }
}
