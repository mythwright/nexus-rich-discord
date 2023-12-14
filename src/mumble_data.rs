use bitmask_enum::bitmask;
use serde::Deserialize;

#[bitmask(u32)]
enum UiState {
    IsMapOpen = 1 << 0,
    IsCompassTopRight = 1 << 1,
    DoesCompassHaveRotationEnabled = 1 << 2,
    GameHasFocus = 1 << 3,
    IsInCompetitiveGameMode = 1 << 4,
    TextboxHasFocus = 1 << 5,
    IsInCombat = 1 << 6,
}

#[derive(Deserialize, Debug)]
struct Identity {
    pub name: String,
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
struct MumbleContext {
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

#[derive(Debug)]
#[repr(C)]
struct MumbleLinkData {
    pub ui_version: u32,
    pub ui_tick: u32,
    pub identity: Identity,
    pub context: MumbleContext,
}