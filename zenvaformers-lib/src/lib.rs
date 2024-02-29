use gdnative::api::Area2D;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Area2D)]
pub struct Tile {
    #[property(default = false)]
    start_tile: bool,
    #[property(default = false)]
    has_building: bool,
    #[property(default = false)]
    can_place_building: bool,
}

#[methods]
impl Tile {
    fn new(_base: &Area2D) -> Self {
        Tile {
            start_tile: false,
            has_building: false,
            can_place_building: false,
        }
    }

    fn get_highlight(base: &Area2D) -> TRef<'static, Sprite> {
        unsafe { base.get_node_as::<Sprite>("Highlight").unwrap() }
    }

    fn get_building_icon(base: &Area2D) -> TRef<'static, Sprite> {
        unsafe { base.get_node_as::<Sprite>("BuildingIcon").unwrap() }
    }

    #[method]
    fn _ready(&self, #[base] base: &Area2D) {
        // godot_print!("Hello from Tile!")
        base.add_to_group("Tiles", false);
    }

    #[method]
    fn toggle_highlight(&mut self, #[base] base: &Area2D, toggle: bool) {
        Self::get_highlight(base).set_visible(toggle);
        self.can_place_building = toggle;
    }

    #[method]
    fn place_building(&mut self, #[base] base: &Area2D, _building_texture: bool) {
        self.has_building = true;
        // TODO: How do I get ahole of building_texture? What is it's type?
        // Self::get_building_icon(base).set_texture(building_texture)
    }

    #[method]
    fn has_building(&mut self, #[base] _base: &Area2D) -> bool {
        self.has_building
    }

    #[method]
    fn _on_tile_input_event(
        &self,
        #[base] base: &Area2D,
        viewport: Ref<Node>,
        event: Ref<InputEvent>,
        shape_idx: i64,
    ) {
        godot_print!("Clicked Tile");
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Map {
    // all_tiles: Vec<TRef<'static, Tile>>,
    // tiles_with_buildings: Vec<TRef<'static, Tile>>,
    #[property(default = 64.0)]
    tile_size: f32,
}

#[methods]
impl Map {
    fn new(_base: &Node) -> Self {
        Map {
            // These could be stored and used with on_ready for efficiency's sake
            // all_tiles: Vec::new(),
            // tiles_with_buildings: Vec::new(),
            tile_size: 64.0,
        }
    }

    #[method]
    fn _ready(&self, #[base] _base: &Node) {
        godot_print!("Hello from Map!")
    }

    #[method]
    fn get_tile_at_position(
        &self,
        #[base] base: &Node,
        position: Vector2,
    ) -> Option<TRef<'static, Area2D>> {
        for tile in base.get_children().iter() {
            let tile = unsafe { tile.try_to_object::<Area2D>().unwrap().assume_safe() };
            let has_building = unsafe { tile.call("has_building", &[]) };
            let has_building: bool = bool::from_variant(&has_building).unwrap();
            if tile.position() == position && !has_building {
                return Some(tile);
            }
        }
        None
    }

    #[method]
    fn disable_tile_highlights(&self, #[base] base: &Node) {
        for tile in base.get_children().iter() {
            let tile = unsafe { tile.try_to_object::<Area2D>().unwrap().assume_safe() };
            unsafe { tile.call("toggle_highlight", &[false.to_variant()]) };
        }
    }

    #[method]
    fn highlight_available_tiles(&self, #[base] base: &Node) {
        // NOTE: In a real app, stuffing these all into vecs at "on_ready" is going to make more sense.
        for tile in base
            .get_children()
            .iter()
            .map(|x| unsafe { x.try_to_object::<Area2D>().unwrap().assume_safe() })
            .filter(|x| bool::from_variant(&unsafe { x.call("has_building", &[]) }).unwrap())
        {
            let north_pos = Vector2::new(tile.position().x, tile.position().y + self.tile_size);
            let south_pos = Vector2::new(tile.position().x, tile.position().y - self.tile_size);
            let east_pos = Vector2::new(tile.position().x + self.tile_size, tile.position().y);
            let west_pos = Vector2::new(tile.position().x - self.tile_size, tile.position().y);

            let north_tile = self.get_tile_at_position(base, north_pos);
            let south_tile = self.get_tile_at_position(base, south_pos);
            let east_tile = self.get_tile_at_position(base, east_pos);
            let west_tile = self.get_tile_at_position(base, west_pos);

            Self::set_highlight_for_optional_tile(north_tile);
            Self::set_highlight_for_optional_tile(south_tile);
            Self::set_highlight_for_optional_tile(east_tile);
            Self::set_highlight_for_optional_tile(west_tile);
        }
    }

    fn set_highlight_for_optional_tile(tile: Option<TRef<'static, Area2D>>) {
        if let Some(t) = tile {
            unsafe { t.call("toggle_highlight", &[true.to_variant()]) };
        };
    }
}

#[derive(NativeClass)]
#[inherit(Control)]
pub struct UI {}

#[methods]
impl UI {
    fn new(_base: &Control) -> Self {
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
    handle.add_class::<Map>();
    handle.add_class::<Tile>();
    handle.add_class::<UI>();
}

// Creates entry-points of dyn lib.
godot_init!(init);
