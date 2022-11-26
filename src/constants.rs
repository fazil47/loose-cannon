use bevy::render::texture::CompressedImageFormats;

pub const CUBEMAP: &(&str, CompressedImageFormats) = &(
    "textures/skybox/corona_skybox.png",
    CompressedImageFormats::NONE,
);
pub const PLANET_SIZE: f32 = 20.0;
pub const PLAYER_SIZE: f32 = 1.0;
pub const FIRE_DELAY: f32 = 2.0; // Delay in seconds until the next cannon can be fired
pub const CAMERA_DISTANCE: f32 = 60.0;
pub const GRAVITY_MAGNITUDE: f32 = 3.0;
pub const PLAYER_IMPULSE_MAGNITUDE: f32 = 200.0;
pub const CANNON_BALL_INITIAL_OFFSET: f32 = 3.0;
pub const SHOW_DEBUG_LINES: bool = false;
