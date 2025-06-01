use godot::classes::AnimatedSprite2D;
use godot::meta::ToGodot;

pub fn handle_player_animation(
    animated_sprite: &mut AnimatedSprite2D,
    direction: f32,
    is_on_floor: bool,
) {
    // Flip the sprite
    if direction > 0.0 {
        animated_sprite.set_flip_h(false);
    } else if direction < 0.0 {
        animated_sprite.set_flip_h(true);
    }

    // Play animations
    if is_on_floor {
        if direction == 0.0 {
            animated_sprite.call("play", &["idle".to_variant()]);
        } else {
            animated_sprite.call("play", &["run".to_variant()]);
        }
    } else {
        animated_sprite.call("play", &["jump".to_variant()]);
    }
}
