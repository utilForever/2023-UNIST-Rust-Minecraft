// Window
pub const OPENGL_MAJOR_VERSION: u32 = 4;
pub const OPENGL_MINOR_VERSION: u32 = 6;
pub const WINDOW_NAME: &str = "Minecraft";
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 800;
pub const NEAR_PLANE: f32 = 0.1;
pub const FAR_PLANE: f32 = 1000.0;
pub const BACKGROUND_COLOR: (f32, f32, f32, f32) = (0.74, 0.84, 1.0, 1.0);

// GUI
pub const CROSSHAIR_SIZE: f32 = 40.0;
pub const BLOCK_OUTLINE_WIDTH: f32 = 3.0;

// Input
pub const MOUSE_SENSITIVITY_X: f32 = 0.5;
pub const MOUSE_SENSITIVITY_Y: f32 = 0.5;

// Physics
pub const PHYSICS_TICKRATE: f32 = 60.0;
pub const GRAVITY: f32 = -28.0;
pub const MAX_VERTICAL_VELOCITY: f32 = 90.0;

// Texture pack
pub const TEXTURE_ATLAS_SIZE: u32 = 1024;
pub const BLOCK_TEXTURE_SIZE: u32 = 16;

// Player
pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYES_HEIGHT: f32 = 1.6;
pub const PLAYER_HALF_WIDTH: f32 = PLAYER_WIDTH / 2.0;
pub const PLAYER_HALF_HEIGHT: f32 = PLAYER_HEIGHT / 2.0;
pub const REACH_DISTANCE: f32 = 400.0;
pub const JUMP_HEIGHT: f32 = 1.3;
pub const HORIZONTAL_ACCELERATION: f32 = 30.0;
pub const WALKING_SPEED: f32 = 4.317;
pub const ON_GROUND_FRICTION: f32 = 12.0;
pub const IN_AIR_FRICTION: f32 = 2.0;

// Calculation of the initial velocity in order to reach the jump height
// NOTE: https://wikimedia.org/api/rest_v1/media/math/render/svg/12be1b7cde89a51c88ef0307f7070cb2368a2079
lazy_static! {
    pub static ref JUMP_IMPULSE: f32 = (JUMP_HEIGHT * 2.0 * -GRAVITY).sqrt();
}
