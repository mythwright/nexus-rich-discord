use bitmask_enum::bitmask;
use serde::Deserialize;

#[bitmask(u32)]
pub enum UiState {
    IsMapOpen = 1 << 0,
    IsCompassTopRight = 1 << 1,
    DoesCompassHaveRotationEnabled = 1 << 2,
    GameHasFocus = 1 << 3,
    IsInCompetitiveGameMode = 1 << 4,
    TextboxHasFocus = 1 << 5,
    IsInCombat = 1 << 6,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[repr(C)]
pub struct Identity {
    pub name: [u8; 20],
    pub profession: u8,
    pub spec: u16,
    pub race: u16,
    pub map_id: usize,
    pub world_id: usize,
    pub team_color_id: usize,
    pub commander: bool,
    pub fov: f64,
    pub uisz: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MumbleContext {
    pub server_address: [u8; 28],
    pub map_id: u32,
    pub map_type: u32,
    pub shard_id: u32,
    pub instance: u32,
    pub build_id: u32,
    pub ui_state: UiState,
    pub compass_width: u16,
    pub compass_height: u16,
    pub compass_rotation: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub map_center_x: f32,
    pub map_center_y: f32,
    pub map_scale: f32,
    pub process_id: u32,
    pub mount_index: u8,
}

/// Three-dimensional Vector
pub type Vector3D = [f32; 3];

#[derive(Copy, Clone, Debug)]
#[repr(C)]
/// Character position in Left hand coordinate system (in SI base units: meters)
pub struct Position {
    /// The character's position in space (in meters).
    pub position: Vector3D,
    /// A unit vector pointing out of the character's eyes (in meters).
    pub front: Vector3D,
    /// A unit vector pointing out of the top of the character's head (in meters).
    pub top: Vector3D,
}

#[derive(Copy, Debug)]
#[repr(C)]
/// MumbleLink data in repr(C) format
pub struct CMumbleLinkData {
    pub ui_version: u32,
    pub ui_tick: u32,
    pub avatar: Position,
    pub name: [i32; 256],
    pub camera: Position,
    pub identity: [u16; 256],
    pub context_len: u32,
    pub context: MumbleContext,
    pub description: [i32; 2048],
}

impl Clone for CMumbleLinkData {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct MumbleLinkData {
    pub ui_version: u32,
    pub ui_tick: u32,
    pub identity: Identity,
    pub context: MumbleContext,
}

#[repr(u8)]
enum EProfession {
    Guardian = 1,
    Warrior = 2,
    Engineer = 3,
    Ranger = 4,
    Thief = 5,
    Elementalist = 6,
    Mesmer = 7,
    Necromancer = 8,
    Revenant = 9,
}

#[repr(u8)]
enum ERace {
    Asura = 0,
    Charr,
    Human,
    Norn,
    Sylvari,
}

#[repr(u8)]
enum EUIScale {
    Small = 0,
    Normal,
    Large,
    Larger,
}

/* structs */
struct CIdentity {
    name: [u8; 20],
    profession: EProfession,
    specialization: u32,
    race: ERace,
    map_id: u32,
    world_id: u32,
    team_color_id: u32,
    is_commander: bool,
    // is the player currently tagged up
    fov: f32,
    uisize: EUIScale,
}