use notify_debouncer_full::DebouncedEvent;

pub fn is_relevant_event(event: &DebouncedEvent) -> bool {
    use notify::event::ModifyKind;
    use notify::EventKind::*;

    match event.kind {
        Create(_) => true,
        Modify(ModifyKind::Data(_)) => true,
        Modify(ModifyKind::Name(_)) => true,
        Remove(_) => true,
        _ => false,
    }
}
pub fn format_event(event: &DebouncedEvent) -> String {
    use notify::event::ModifyKind;
    use notify::EventKind::*;

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
