use crate::{create_effect, create_signal, Signal};
use crate::animation::request_animation_frame;

/// Creates a new signal that interpolates towards the target value over a given duration.
pub fn use_tween(mut target: impl FnMut() -> f32 + 'static, duration: f32, easing: fn(f32) -> f32) -> Signal<f32> {
    let current = create_signal(target());
    
    // We need to keep track of state across animation frames. 
    // We shouldn't use signals for this internal state because we don't want subscribers to react 
    // to internal timer changes, just the value changes.
    // Instead, we capture a mutable variable in the effect.
    // Wait, `create_effect` re-runs entirely when `target` changes. So we can just initialize state here!
    
    let anim_id = std::rc::Rc::new(std::cell::Cell::new(0usize));
    
    create_effect(move || {
        let dest = target();
        let current_id = anim_id.get() + 1;
        anim_id.set(current_id);
        
        let start_val = match current.try_get() {
            Some(v) => v,
            None => return, // disposed
        };
        
        let mut elapsed = 0.0_f32;
        let anim_id_clone = anim_id.clone();
        
        request_animation_frame(move |dt| {
            if anim_id_clone.get() != current_id {
                return false; // superseded
            }
            
            elapsed += dt;
            if elapsed >= duration {
                current.set(dest);
                false // stop animation
            } else {
                let progress = elapsed / duration;
                let eased = easing(progress);
                current.set(start_val + (dest - start_val) * eased);
                true // keep running
            }
        });
    });
    
    current
}
