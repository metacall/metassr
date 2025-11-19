use notify::event::ModifyKind;
use notify::EventKind::*;
use notify_debouncer_full::DebouncedEvent;

pub fn is_relevant_event(event: &DebouncedEvent) -> bool {
    matches!(
        event.kind,
        Create(_) | Modify(ModifyKind::Data(_)) | Modify(ModifyKind::Name(_)) | Remove(_)
    )
}
pub fn format_event(event: &DebouncedEvent) -> String {
    let action = match event.kind {
        Create(_) => "created",
        Modify(ModifyKind::Data(_)) => "modified",
        Modify(ModifyKind::Name(_)) => "renamed",
        Remove(_) => "deleted",
        _ => "unknown action",
    };

    let paths = event
        .paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!("{action} {paths}")
}
