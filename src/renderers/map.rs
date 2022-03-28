/// Renders the ground colours.
pub mod base;
/// Responsible for drawing lines - doors, fences, walls and so on.
pub mod lines;
/// Describes the shape of lines drawn by the map renderer.
pub mod lineshape;
/// Core of the render - this is where everything is put together.
pub mod mapcore;
/// Responsible for drawing [`MapScene`](crate::definitions::mapscenes::MapScene).
pub mod mapscenes;
/// Describes the shape of overlays drawn by the map renderer.
pub mod tileshape;

pub use self::mapcore::{render, save_smallest};
