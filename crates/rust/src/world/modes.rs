use godot::classes::{InputEvent, InputEventMouseButton, InputEventMouseMotion, TileMapLayer};
use godot::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorldMode {
    #[default]
    None,
    Edit,
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct ModesManager {
    tilemap_layer: Option<Gd<TileMapLayer>>,
    hovered_cell: Vector2i,

    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for ModesManager {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            tilemap_layer: None,
            hovered_cell: Vector2i::new(-9_999_999, -9_999_999),
        }
    }

    fn ready(&mut self) {
        // Try to find the main TileMapLayer near this node.
        // Common placements: sibling or direct child of the scene root.
        self.tilemap_layer = Some(
            self.base()
                .try_get_node_as::<TileMapLayer>("TileMapLayer")
                .expect("Failed to find TileMapLayer"),
        );
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(event) = event.clone().try_cast::<InputEventMouseButton>() {
            let pos = event.get_position();
            godot_print!("click_pos: {:?}, {}", pos, event.is_pressed());
            return;
        }

        if let Ok(_event) = event.try_cast::<InputEventMouseMotion>() {
            let Some(ref tilemap) = self.tilemap_layer else {
                return;
            };

            // Use the tilemap's local mouse position so camera/canvas transforms are respected.
            // Then convert local coords -> map cell.
            let local_pos: Vector2 = tilemap.get_local_mouse_position();
            let cell: Vector2i = tilemap.local_to_map(local_pos);

            godot_print!("cell: {:?}", cell);

            if cell != self.hovered_cell {
                self.hovered_cell = cell;
                self.base_mut().queue_redraw();
            }
        }
    }

    fn draw(&mut self) {
        let Some(ref tilemap) = self.tilemap_layer else {
            return;
        };

        // Convert the hovered cell back to local pixel-space center.
        let center: Vector2 = tilemap.map_to_local(self.hovered_cell);

        // Derive tile size by sampling neighbor cell centers (robust even without direct tileset read).
        let right_center: Vector2 = tilemap.map_to_local(self.hovered_cell + Vector2i::new(1, 0));
        let down_center: Vector2 = tilemap.map_to_local(self.hovered_cell + Vector2i::new(0, 1));
        let tile_w: f32 = (right_center.x - center.x).abs();
        let tile_h: f32 = (down_center.y - center.y).abs();

        // Fallbacks in case sampling fails (e.g., unusual transforms).
        let w = if tile_w.is_finite() && tile_w > 0.0 {
            tile_w
        } else {
            16.0
        };
        let h = if tile_h.is_finite() && tile_h > 0.0 {
            tile_h
        } else {
            16.0
        };

        let origin = center - Vector2::new(w * 0.5, h * 0.5);
        let rect = Rect2::new(origin, Vector2::new(w, h));

        // Fill + outline similar to the GDScript version.
        self.base_mut()
            .draw_rect(rect, Color::from_rgba(1.0, 1.0, 0.5, 0.22));
        self.base_mut()
            .draw_rect(rect, Color::from_rgba(1.0, 1.0, 0.5, 0.90));
    }
}
