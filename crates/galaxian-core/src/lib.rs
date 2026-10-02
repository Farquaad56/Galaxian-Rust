//! Galaxian core — bus, timing, memory, I/O, video, orchestration (Partie C).
//!
//! Skeleton only; modules are filled in by later tasks (Partie F, T2.x–T4.x).

#![forbid(unsafe_code)]

/// Memory map and address decoding.
pub mod memory;

/// Time types: Z80 cycles (T-states) and hardware timing constants (KB-02).
pub mod time;
