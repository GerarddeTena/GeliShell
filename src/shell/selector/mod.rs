pub mod modal;

use crate::shell::translator::resolver::ResolvedCommand;

/// Resultado de la selección interactiva
#[derive(Debug, Clone)]
pub enum SelectionResult {
    /// El usuario eligió esta opción
    Selected(String),
    /// El usuario canceló con Esc
    Cancelled,
}

/// Contrato del selector — Open/Closed para distintas presentaciones
pub trait CommandSelector: Send + Sync {
    fn select(&self, resolved: &ResolvedCommand) -> SelectionResult;
}
