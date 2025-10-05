use notify_debouncer_full::DebouncedEvent;

pub fn is_relevant_event(event: &DebouncedEvent) -> bool {
    use notify::event::ModifyKind;
    use notify::EventKind::*;

    // Filter out temporary files and directories
    if event.paths.iter().any(|p| {
        p.to_string_lossy().contains(".swp") || p.to_string_lossy().contains(".tmp") || p.is_dir()
    }) {
        return false;
    }

    match event.kind {
        Create(_) | Modify(ModifyKind::Data(_)) | Modify(ModifyKind::Name(_)) | Remove(_) => true,
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
