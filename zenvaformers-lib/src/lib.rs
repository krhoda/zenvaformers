use gdnative::{
    api::Area2D,
    export::{
        hint::{EnumHint, IntHint},
        Export,
    },
    prelude::*,
};

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
    fn is_start_tile(&self, #[base] _base: &Area2D) -> bool {
        self.start_tile
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
    fn place_building(&mut self, #[base] base: &Area2D, building_type: BuildingType) {
        godot_print!("In place building");
        self.has_building = true;
        let t = load::<Texture>(building_type.get_texture_path()).unwrap();
        Self::get_building_icon(base).set_texture(t)
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
        // godot_print!("Clicked Tile");
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
    fn _ready(&self, #[base] base: &Node) {
        godot_print!("Hello from Map!");
        // Should create all_tiles here. not convinced tiles_with_buildings is needed.
        for tile in base.get_children().iter() {
            let tile = unsafe { tile.try_to_object::<Area2D>().unwrap().assume_safe() };
            let is_start_tile = unsafe { tile.call("is_start_tile", &[]) };
            let is_start_tile = bool::from_variant(&is_start_tile).unwrap();
            if is_start_tile {
                godot_print!("Found start_tile");
                unsafe { tile.call("place_building", &[BuildingType::Base.to_variant()]) };
                break;
            }
        }
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

    #[method]
    fn place_building(&self, #[base] base: &Node, tile_position: Vector2, building_type: Building) {
        let tile = self.get_tile_at_position(base, tile_position).unwrap();
        unsafe { tile.call("place_building", &[building_type.to_variant()]) };
        self.disable_tile_highlights(base);
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

#[derive(Clone, PartialEq)]
pub enum BuildingType {
    Base = 0,
    Mine = 1,
    Greenhouse = 2,
    SolarPanel = 3,
}

impl BuildingType {
    pub fn get_texture_path(&self) -> String {
        match self {
            BuildingType::Base => "res://Sprites/Base.png".to_string(),
            BuildingType::Mine => "res://Sprites/Mine.png".to_string(),
            BuildingType::Greenhouse => "res://Sprites/Greenhouse.png".to_string(),
            BuildingType::SolarPanel => "res://Sprites/SolarPanel.png".to_string(),
        }
    }
}

impl ToVariant for BuildingType {
    fn to_variant(&self) -> Variant {
        match self {
            BuildingType::Base => 0.to_variant(),
            BuildingType::Mine => 1.to_variant(),
            BuildingType::Greenhouse => 2.to_variant(),
            BuildingType::SolarPanel => 3.to_variant(),
        }
    }
}

impl FromVariant for BuildingType {
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        let result = i64::from_variant(variant)?;
        match result {
            0 => Ok(BuildingType::Base),
            1 => Ok(BuildingType::Mine),
            2 => Ok(BuildingType::Greenhouse),
            3 => Ok(BuildingType::SolarPanel),
            _ => Err(FromVariantError::UnknownEnumVariant {
                variant: "i64".to_owned(),
                expected: &["0", "1", "2", "3"],
            }),
        }
    }
}

impl Export for BuildingType {
    type Hint = IntHint<u32>;

    fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
        Self::Hint::Enum(EnumHint::new(vec![
            "Base".to_owned(),
            "Mine".to_owned(),
            "Greenhouse".to_owned(),
            "SolarPanel".to_owned(),
        ]))
        .export_info()
    }
}

#[derive(Clone, PartialEq)]
pub enum ResourceType {
    Nothing = 0,
    Food = 1,
    Metal = 2,
    Oxygen = 3,
    Energy = 4,
}

impl ToVariant for ResourceType {
    fn to_variant(&self) -> Variant {
        match self {
            ResourceType::Nothing => 0.to_variant(),
            ResourceType::Food => 1.to_variant(),
            ResourceType::Metal => 2.to_variant(),
            ResourceType::Oxygen => 3.to_variant(),
            ResourceType::Energy => 4.to_variant(),
        }
    }
}

impl FromVariant for ResourceType {
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        let result = i64::from_variant(variant)?;
        match result {
            0 => Ok(ResourceType::Nothing),
            1 => Ok(ResourceType::Food),
            2 => Ok(ResourceType::Metal),
            3 => Ok(ResourceType::Oxygen),
            4 => Ok(ResourceType::Energy),
            _ => Err(FromVariantError::UnknownEnumVariant {
                variant: "i64".to_owned(),
                expected: &["0", "1", "2", "3"],
            }),
        }
    }
}

impl Export for ResourceType {
    type Hint = IntHint<u32>;

    fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
        Self::Hint::Enum(EnumHint::new(vec![
            "Nothing".to_owned(),
            "Food".to_owned(),
            "Metal".to_owned(),
            "Oxygen".to_owned(),
            "Energy".to_owned(),
        ]))
        .export_info()
    }
}

#[derive(NativeClass, ToVariant, FromVariant)]
#[inherit(Node)]
pub struct Building {
    #[property]
    building_type: BuildingType,
    #[property]
    resource_type: ResourceType,
    #[property]
    resource_amount: i32,
    #[property]
    upkeep_type: ResourceType,
    #[property]
    upkeep_amount: i32,
}

#[methods]
impl Building {
    fn new(_base: &Node) -> Self {
        Building {
            building_type: BuildingType::Base,
            resource_amount: 0,
            resource_type: ResourceType::Nothing,
            upkeep_amount: 0,
            upkeep_type: ResourceType::Nothing,
        }
    }

    fn base() -> Self {
        Building {
            building_type: BuildingType::Base,
            resource_amount: 0,
            resource_type: ResourceType::Nothing,
            upkeep_amount: 0,
            upkeep_type: ResourceType::Nothing,
        }
    }

    fn mine() -> Self {
        Building {
            building_type: BuildingType::Mine,
            resource_amount: 1,
            resource_type: ResourceType::Metal,
            upkeep_amount: 1,
            upkeep_type: ResourceType::Energy,
        }
    }

    fn greenhouse() -> Self {
        Building {
            building_type: BuildingType::Greenhouse,
            resource_amount: 1,
            resource_type: ResourceType::Food,
            upkeep_amount: 0,
            upkeep_type: ResourceType::Nothing,
        }
    }

    fn solar_panel() -> Self {
        Building {
            building_type: BuildingType::SolarPanel,
            resource_amount: 1,
            resource_type: ResourceType::Energy,
            upkeep_amount: 0,
            upkeep_type: ResourceType::Nothing,
        }
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct BuildingData {}

#[methods]
impl BuildingData {
    fn new(_base: &Node) -> Self {
        BuildingData {}
    }

    // The idea is any other game object passes in _their_ base to grab _this_ node
    pub fn get_singleton_node(base: &Node) -> TRef<'static, Node> {
        unsafe {
            base.get_node_as::<Node>("root/MainScene/BuildingData")
                .unwrap()
        }
    }

    #[method]
    fn _ready(&self, #[base] _base: &Node) {
        godot_print!("Hello from Building Data!")
    }

    #[method]
    fn data(&self, #[base] _base: &Node, building_type: BuildingType) -> Building {
        match building_type {
            BuildingType::Base => Building::base(),
            BuildingType::Mine => Building::mine(),
            BuildingType::Greenhouse => Building::greenhouse(),
            BuildingType::SolarPanel => Building::solar_panel(),
        }
    }
}

// use godot_sane_defaults::kb2d_move_and_slide;
// Registers all exposed classes to Godot.
fn init(handle: InitHandle) {
    handle.add_class::<Building>();
    handle.add_class::<BuildingData>();
    handle.add_class::<Map>();
    handle.add_class::<Tile>();
    handle.add_class::<UI>();
}

// Creates entry-points of dyn lib.
godot_init!(init);
