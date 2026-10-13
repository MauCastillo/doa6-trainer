//! Casos de uso del trainer. Depende del dominio y de los puertos, nunca del SO.

pub mod controller;
pub mod error;
pub mod patcher;
pub mod ports;
pub mod steam_emu;
pub mod trainer;

#[cfg(test)]
mod fake_memory;
