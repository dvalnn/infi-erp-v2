// MODULES
mod bom;
mod orders;

// RE-EXPORTS
pub use bom::*;
pub use orders::*;

pub enum NotificationChannel {
    NewOrder,
    NewBomEntry,
    Unknown,
}

impl NotificationChannel {
    const NEW_ORDER_CHANNEL: &'static str = "new_order";
    const NEW_BOM_ENTRY_CHANNEL: &'static str = "new_bom_entry";
    pub const ALL_STR: [&'static str; 2] =
        [Self::NEW_ORDER_CHANNEL, Self::NEW_BOM_ENTRY_CHANNEL];
}

impl std::fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NotificationChannel as Nc;
        match self {
            Nc::NewOrder => write!(f, "new_order"),
            Nc::NewBomEntry => write!(f, "new_bom_entry"),
            Nc::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for NotificationChannel {
    fn from(s: &str) -> Self {
        use NotificationChannel as Nc;
        match s {
            Nc::NEW_ORDER_CHANNEL => Nc::NewOrder,
            Nc::NEW_BOM_ENTRY_CHANNEL => Nc::NewBomEntry,
            _ => Nc::Unknown,
        }
    }
}
