use crate::{create_effect, create_signal, Signal};
use crate::animation::request_animation_frame;

#[derive(Clone, Copy, Debug)]
pub struct SpringConfig {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
    pub precision: f32,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: 1500.0,
            damping: 40.0,
            mass: 1.0,
            precision: 0.001,
        }
    }
}

impl SpringConfig {
    pub fn bouncy() -> Self {
        Self { stiffness: 2000.0, damping: 30.0, mass: 1.0, precision: 0.001 }
    }
    
    pub fn stiff() -> Self {
        Self { stiffness: 2500.0, damping: 50.0, mass: 1.0, precision: 0.001 }
    }
    
    pub fn sluggish() -> Self {
        Self { stiffness: 500.0, damping: 35.0, mass: 1.0, precision: 0.001 }
    }
}

/// Creates a new signal that springs towards the target value over time.
pub fn use_spring(mut target: impl FnMut() -> f32 + 'static, config: SpringConfig) -> Signal<f32> {
    let current = create_signal(target());
    let velocity = create_signal(0.0_f32);
    
    // We use a separate state cell for the animation loop so we don't have to read/write signals continuously 
    // inside the hot animation loop which might trigger repaints needlessly if not bounded.
    // Wait, if we use signals, setting it WILL trigger a repaint, which is what we want!
    
    let anim_id = std::rc::Rc::new(std::cell::Cell::new(0usize));
    
    // Track the target. Whenever it changes, ensure the animation loop is running.
    create_effect(move || {
        let dest = target();
        let current_id = anim_id.get() + 1;
        anim_id.set(current_id);
        
        let anim_id_clone = anim_id.clone();
        
        request_animation_frame(move |dt| {
            if anim_id_clone.get() != current_id {
                return false; // Superseded by a newer animation
            }
            
            let (mut c, mut v) = match (current.try_get(), velocity.try_get()) {
                (Some(c), Some(v)) => (c, v),
                _ => return false, // Signals disposed, stop animation
            };
            
            // Sub-step the integration to ensure stability even with large dt
            let max_step = 0.008; // 8ms maximum step
            let mut remaining_dt = dt;
            
            while remaining_dt > 0.0 {
                let step = remaining_dt.min(max_step);
                remaining_dt -= step;
                
                let force = -config.stiffness * (c - dest) - config.damping * v;
                let acceleration = force / config.mass;
                
                v += acceleration * step;
                c += v * step;
            }
            
            current.set(c);
            velocity.set(v);
            
            let is_moving = v.abs() > config.precision;
            let is_distant = (c - dest).abs() > config.precision;
            
            if !is_moving && !is_distant {
                // Snap exactly to target when resting
                current.set(dest);
                velocity.set(0.0);
                false // stop animation
            } else {
                true // keep running
            }
        });
    });
    
    current
}
