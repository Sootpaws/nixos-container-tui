pub use manager::ManagerProxy;
pub use unit::UnitProxy;

/// Main systemd manager interface
mod manager;

/// Interface for systemd units
mod unit;
