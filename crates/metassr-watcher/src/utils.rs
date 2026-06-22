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

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, RemoveKind};
    use notify::EventKind;
    use std::path::PathBuf;
    use std::time::Instant;

    fn make_event(kind: EventKind, paths: Vec<PathBuf>) -> DebouncedEvent {
        let event = notify::Event {
            kind,
            paths,
            attrs: Default::default(),
        };
        DebouncedEvent::new(event, Instant::now())
    }

    #[test]
    fn accepts_file_create_events() {
        let event = make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("src/pages/index.tsx")],
        );
        assert!(is_relevant_event(&event));
    }

    #[test]
    fn accepts_data_modify_events() {
        let event = make_event(
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            vec![PathBuf::from("src/pages/about.tsx")],
        );
        assert!(is_relevant_event(&event));
    }

    #[test]
    fn accepts_rename_events() {
        let event = make_event(
            EventKind::Modify(ModifyKind::Name(notify::event::RenameMode::Both)),
            vec![
                PathBuf::from("src/pages/old.tsx"),
                PathBuf::from("src/pages/new.tsx"),
            ],
        );
        assert!(is_relevant_event(&event));
    }

    #[test]
    fn accepts_remove_events() {
        let event = make_event(
            EventKind::Remove(RemoveKind::File),
            vec![PathBuf::from("src/pages/deleted.tsx")],
        );
        assert!(is_relevant_event(&event));
    }

    #[test]
    fn ignores_access_events() {
        let event = make_event(
            EventKind::Access(notify::event::AccessKind::Read),
            vec![PathBuf::from("src/pages/index.tsx")],
        );
        assert!(!is_relevant_event(&event));
    }

    #[test]
    fn ignores_metadata_modify_events() {
        let event = make_event(
            EventKind::Modify(ModifyKind::Metadata(
                notify::event::MetadataKind::Permissions,
            )),
            vec![PathBuf::from("src/pages/index.tsx")],
        );
        assert!(!is_relevant_event(&event));
    }
}
