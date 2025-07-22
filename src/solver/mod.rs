// src/solver/mod.rs
pub mod g2p;
pub mod p2g;
pub mod particle;

// Re-export from the solver module
pub use g2p::*;
pub use p2g::*;
pub use particle::*;

// Create a prelude module for easy imports
pub mod prelude {
    pub use super::g2p::*;
    pub use super::p2g::*;
    pub use super::particle::*;
}
