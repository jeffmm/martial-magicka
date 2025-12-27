pub mod idle;
pub mod movement;
pub mod jump;
pub mod punch;
pub mod kick;
pub mod combo;
pub mod aerial;
pub mod defeat;

// Re-export state data structs
pub use idle::*;
pub use movement::*;
pub use jump::*;
pub use punch::*;
pub use kick::*;
pub use combo::*;
pub use aerial::*;
pub use defeat::*;
