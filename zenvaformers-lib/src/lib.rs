use gdnative::{
    api::{Area2D, HBoxContainer, InputEventMouseButton},
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
        &mut self,
        #[base] base: &Area2D,
        viewport: Ref<Node>,
        event: Ref<InputEvent>,
        shape_idx: i64,
    ) {
        let event = unsafe { event.assume_safe() };
        if let Some(event) = event.cast::<InputEventMouseButton>() {
            if event.is_pressed() {
                let game_manager = unsafe { base.get_node_as::<Node>("/root/MainScene").unwrap() };
                let state = unsafe { game_manager.call("state", &[]) };
                let state = GameManager::from_variant(&state).unwrap();

                if self.can_place_building && state.is_placing_building {
                    self.place_building(base, state.building_to_place);
                    unsafe { game_manager.call("place_building", &[base.position().to_variant()]) };
                };
            }
        }
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
    fn get_tile_path(&self, #[base] base: &Node, position: Vector2) -> Option<String> {
        self.get_tile_at_position(base, position)
            .map(|x| x.get_path().to_string())
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
            // NOTE: Inelligant hacking below, and the variable names lie too!
            // NOTE: Tiles need to be perfectly placed.
            let north_pos = Vector2::new(
                tile.position().x,
                tile.position().y - (self.tile_size - 8.0),
            );
            let south_pos = Vector2::new(
                tile.position().x,
                tile.position().y + (self.tile_size - 2.0),
            );
            let east_pos = Vector2::new(tile.position().x + self.tile_size, tile.position().y);
            let west_pos = Vector2::new(tile.position().x - self.tile_size, tile.position().y);

            // Above was a mess, time for good old fashioned hard coding!
            let north_tile = self.get_tile_at_position(base, north_pos);
            let south_tile = self.get_tile_at_position(base, south_pos);
            let east_tile = self.get_tile_at_position(base, east_pos);
            let west_tile = self.get_tile_at_position(base, west_pos);

            godot_print!(
                "{:?}, {:?}, {:?}, {:?}",
                north_pos,
                south_pos,
                east_pos,
                west_pos
            );

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
    fn place_building(
        &self,
        #[base] base: &Node,
        tile_position: Vector2,
        building_type: BuildingType,
    ) {
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
    fn get_building_buttons(&self, #[base] base: &Control) -> TRef<'static, HBoxContainer> {
        unsafe {
            base.get_node_as::<HBoxContainer>("BuildingButtons")
                .unwrap()
        }
    }

    #[method]
    fn get_food_metal_text(&self, #[base] base: &Control) -> TRef<'static, Label> {
        unsafe { base.get_node_as::<Label>("FoodMetalText").unwrap() }
    }

    #[method]
    fn get_oxygen_energy_text(&self, #[base] base: &Control) -> TRef<'static, Label> {
        unsafe { base.get_node_as::<Label>("OxygenEnergyText").unwrap() }
    }

    #[method]
    fn get_turn_text(&self, #[base] base: &Control) -> TRef<'static, Label> {
        unsafe { base.get_node_as::<Label>("TurnText").unwrap() }
    }

    #[method]
    fn get_game_manager(&self, #[base] base: &Control) -> TRef<'static, Node> {
        unsafe { base.get_node_as::<Node>("/root/MainScene").unwrap() }
    }

    #[method]
    fn _ready(&self, #[base] _base: &Control) {
        godot_print!("Hello from UI!")
    }

    #[method]
    fn on_end_turn(&self, #[base] base: &Control, state: GameManager) {
        let turn_text = self.get_turn_text(base);
        turn_text.set_text(format!("Turn: {}", state.turn_number));

        let building_buttons = self.get_building_buttons(base);
        building_buttons.set_visible(true);

        let next_food_operator = if state.income_food >= 0 { "+" } else { "-" };
        let next_metal_operator = if state.income_metal >= 0 { "+" } else { "-" };
        let next_oxygen_operator = if state.income_oxygen >= 0 { "+" } else { "-" };
        let next_energy_operator = if state.income_energy >= 0 { "+" } else { "-" };

        let next_food_metal = format!(
            "{} ({}{})\n{} ({}{})",
            state.current_food,
            next_food_operator,
            state.income_food,
            state.current_metal,
            next_metal_operator,
            state.income_metal
        );
        let next_oxygen_energy = format!(
            "{} ({}{})\n{} ({}{})",
            state.current_oxygen,
            next_oxygen_operator,
            state.income_oxygen,
            state.current_energy,
            next_energy_operator,
            state.income_energy
        );

        let food_metal = self.get_food_metal_text(base);
        food_metal.set_text(next_food_metal);

        let oxygen_energy = self.get_oxygen_energy_text(base);
        oxygen_energy.set_text(next_oxygen_energy);
    }

    #[method]
    fn _on_end_turn_button_pressed(&self, #[base] base: &Control) {
        let game_manager = self.get_game_manager(base);
        unsafe { game_manager.call("end_turn", &[]) };
        let state = unsafe { game_manager.call("state", &[]) };
        let state = GameManager::from_variant(&state).unwrap();
        self.on_end_turn(base, state);
    }

    #[method]
    fn _on_mine_button_pressed(&self, #[base] base: &Control) {
        let building_buttons = self.get_building_buttons(base);
        building_buttons.set_visible(false);

        let game_manager = self.get_game_manager(base);
        unsafe { game_manager.call("on_select_building", &[BuildingType::Mine.to_variant()]) };
    }

    #[method]
    fn _on_greenhouse_button_pressed(&self, #[base] base: &Control) {
        let building_buttons = self.get_building_buttons(base);
        building_buttons.set_visible(false);

        let game_manager = self.get_game_manager(base);
        unsafe {
            game_manager.call(
                "on_select_building",
                &[BuildingType::Greenhouse.to_variant()],
            )
        };
    }

    #[method]
    fn _on_solar_panel_button_pressed(&self, #[base] base: &Control) {
        godot_print!("In solar panel button press");
        let building_buttons = self.get_building_buttons(base);
        godot_print!("Have building buttons");
        building_buttons.set_visible(false);

        let game_manager = self.get_game_manager(base);
        unsafe {
            game_manager.call(
                "on_select_building",
                &[BuildingType::SolarPanel.to_variant()],
            )
        };
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
            base.get_node_as::<Node>("/root/MainScene/BuildingData")
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

#[derive(NativeClass, Clone, ToVariant, FromVariant)]
#[inherit(Node2D)]
pub struct GameManager {
    #[property(default = 0)]
    current_food: i32,
    #[property(default = 0)]
    current_metal: i32,
    #[property(default = 0)]
    current_oxygen: i32,
    #[property(default = 0)]
    current_energy: i32,
    #[property(default = 0)]
    income_food: i32,
    #[property(default = 0)]
    income_metal: i32,
    #[property(default = 0)]
    income_oxygen: i32,
    #[property(default = 0)]
    income_energy: i32,
    #[property(default = 1)]
    turn_number: i32,
    #[property(default = false)]
    is_placing_building: bool,
    #[property]
    building_to_place: BuildingType,
}

#[methods]
impl GameManager {
    fn new(_base: &Node2D) -> Self {
        GameManager {
            current_food: 0,
            current_metal: 0,
            current_oxygen: 0,
            current_energy: 0,
            income_food: 0,
            income_metal: 0,
            income_oxygen: 0,
            income_energy: 0,
            turn_number: 1,
            is_placing_building: false,
            building_to_place: BuildingType::Base,
        }
    }

    #[method]
    fn state(&self, #[base] _base: &Node2D) -> Self {
        self.clone()
    }

    #[method]
    fn _ready(&self, #[base] base: &Node2D) {
        godot_print!("Hello from Game Manager!");
        let ui = unsafe { base.get_node_as::<Control>("UI").unwrap() };
        unsafe { ui.call("update_resource_text", &[]) };
        unsafe { ui.call("on_end_turn", &[]) };
    }

    #[method]
    fn on_select_building(&mut self, #[base] base: &Node2D, building_type: BuildingType) {
        self.is_placing_building = true;
        self.building_to_place = building_type;
        let map = unsafe { base.get_node_as::<Node>("Tiles").unwrap() };
        unsafe { map.call("highlight_available_tiles", &[]) };
    }

    #[method]
    fn add_to_resource_per_turn(
        &mut self,
        #[base] _base: &Node2D,
        resource_type: ResourceType,
        amount: i32,
    ) {
        match resource_type {
            ResourceType::Energy => self.income_energy += amount,
            ResourceType::Food => self.income_food += amount,
            ResourceType::Metal => self.income_metal += amount,
            ResourceType::Oxygen => self.income_oxygen += amount,
            ResourceType::Nothing => {}
        }
    }

    #[method]
    fn place_building(&mut self, #[base] base: &Node2D, tile_position: Vector2) {
        let b = unsafe {
            BuildingData::get_singleton_node(base)
                .call("data", &[self.building_to_place.to_variant()])
        };

        let b = Building::from_variant(&b).unwrap();
        self.add_to_resource_per_turn(base, b.resource_type, b.resource_amount);
    }

    #[method]
    fn end_turn(&mut self, #[base] base: &Node2D) {
        self.current_energy += self.income_energy;
        self.current_food += self.income_food;
        self.current_metal += self.income_metal;
        self.current_oxygen += self.current_oxygen;
        self.turn_number += 1;

        // let ui = unsafe { base.get_node_as::<Control>("UI").unwrap() };
        // unsafe { ui.call("update_resource_text", &[self.state().to_variant()]) };
        // unsafe { ui.call("on_end_turn", &[self.state().to_variant()]) };
    }
}
// use godot_sane_defaults::kb2d_move_and_slide;
// Registers all exposed classes to Godot.
fn init(handle: InitHandle) {
    handle.add_class::<Building>();
    handle.add_class::<BuildingData>();
    handle.add_class::<GameManager>();
    handle.add_class::<Map>();
    handle.add_class::<Tile>();
    handle.add_class::<UI>();
}

// Creates entry-points of dyn lib.
godot_init!(init);
