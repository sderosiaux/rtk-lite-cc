//! Rewrite infrastructure for command transformation (used by hooks).
//!
//! This module provides core components for analyzing and transforming
//! shell commands, primarily used by RTK's hook system (Gemini CLI, Copilot, etc.).

pub mod lexer;
pub mod registry;
pub mod rules;
