//! Built-in plugins: Help, History, and Alias.

mod alias;
mod help;
mod history;

pub use alias::AliasPlugin;
pub use help::HelpPlugin;
pub use history::HistoryPlugin;
