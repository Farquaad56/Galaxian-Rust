//! Z80 CPU core — cycle-accurate, no_std-compatible, hardware-agnostic (Partie C).
//!
//! Skeleton only; modules are filled in by later tasks (Partie F, T1.x).

#![no_std]

/// Arithmetic and logic unit.
pub mod alu;

/// Register file and flags.
pub mod regs;

/// T-state timing tables.
pub mod timing;
