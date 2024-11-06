use colored::{ColoredString, Colorize};
use serde_with::SerializeDisplay;

#[derive(SerializeDisplay, Debug, Clone, Default)]
pub(crate) enum StatusIcon {
    Succeeded,
    Failed,
    Cancelled,
    Queued,
    #[default]
    Warning,
}

impl From<&StatusIcon> for ColoredString {
    fn from(icon: &StatusIcon) -> Self {
        match icon {
            StatusIcon::Succeeded => "✔".green(),
            StatusIcon::Failed => "✖".red(),
            StatusIcon::Cancelled => "⏹".red(),
            StatusIcon::Queued => "⧖".yellow(),
            StatusIcon::Warning => "⚠".yellow(),
        }
    }
}

impl std::fmt::Display for StatusIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = ColoredString::from(self).normal();
        write!(f, "{icon}")
    }
}

#[cfg(test)]
use insta::assert_snapshot;

#[test]
fn serialize_and_colorize_icons() {
    let success_icon = serde_json::to_string(&StatusIcon::Succeeded).unwrap();
    debug_assert_eq!(success_icon, r#""✔""#);
    let colored_icon = ColoredString::from(&StatusIcon::Queued);
    assert_snapshot!(colored_icon.to_string(), @"[33m⧖[0m");
}
