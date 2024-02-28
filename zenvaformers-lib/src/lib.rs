// use gdnative::api::{Area2D, AudioStream, AudioStreamPlayer2D, Camera2D};
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct UI {}

#[methods]
impl UI {
    fn new(base: &Control) -> Self {
        UI {}
    }

    #[method]
    fn _ready(&self, #[base] _base: &Control) {
        godot_print!("Hello from UI!")
    }
}

// use godot_sane_defaults::kb2d_move_and_slide;
// Registers all exposed classes to Godot.
fn init(handle: InitHandle) {
    handle.add_class::<UI>();
}

// Creates entry-points of dyn lib.
godot_init!(init);
