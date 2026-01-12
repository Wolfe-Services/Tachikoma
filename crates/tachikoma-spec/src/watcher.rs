use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Events from spec directory watching
#[derive(Debug, Clone)]
pub enum SpecDirectoryEvent {
    SpecCreated(PathBuf),
    SpecModified(PathBuf),
    SpecDeleted(PathBuf),
    PhaseCreated(PathBuf),
    PhaseDeleted(PathBuf),
    ConfigChanged,
    TemplateChanged(PathBuf),
}

/// Watch spec directory for changes
pub struct SpecDirectoryWatcher {
    _watcher: notify::RecommendedWatcher,
    receiver: mpsc::Receiver<SpecDirectoryEvent>,
}

impl SpecDirectoryWatcher {
    pub fn new(spec_root: PathBuf) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if let Some(spec_event) = Self::classify_event(&event, &spec_root) {
                    let _ = tx.blocking_send(spec_event);
                }
            }
        })?;

        watcher.watch(&spec_root, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    fn classify_event(event: &Event, _root: &PathBuf) -> Option<SpecDirectoryEvent> {
        let path = event.paths.first()?;

        match &event.kind {
            EventKind::Create(_) => {
                if path.is_dir()
                    && path
                        .file_name()?
                        .to_str()?
                        .starts_with("phase-")
                {
                    Some(SpecDirectoryEvent::PhaseCreated(path.clone()))
                } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    Some(SpecDirectoryEvent::SpecCreated(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Modify(_) => {
                if path.file_name()?.to_str()? == "specs.toml" {
                    Some(SpecDirectoryEvent::ConfigChanged)
                } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    Some(SpecDirectoryEvent::SpecModified(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Remove(_) => {
                if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    Some(SpecDirectoryEvent::SpecDeleted(path.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub async fn next_event(&mut self) -> Option<SpecDirectoryEvent> {
        self.receiver.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SpecDirectory;
    use tempfile::TempDir;
    use tokio::fs;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_watcher_detects_spec_creation() {
        let temp = TempDir::new().unwrap();
        let dir = SpecDirectory::initialize(temp.path()).await.unwrap();

        let mut watcher = SpecDirectoryWatcher::new(dir.root.clone()).unwrap();

        // Create a phase directory
        let phase_dir = dir.phase_path(6);
        fs::create_dir_all(&phase_dir).await.unwrap();

        // Give watcher time to detect
        sleep(Duration::from_millis(100)).await;

        // Create a spec file
        let spec_file = phase_dir.join("116-test-spec.md");
        fs::write(&spec_file, "# Test Spec").await.unwrap();

        // Give watcher time to detect
        sleep(Duration::from_millis(100)).await;

        // Check if we get the expected events
        if let Some(event) = watcher.next_event().await {
            match event {
                SpecDirectoryEvent::PhaseCreated(path) => {
                    assert!(path.to_string_lossy().contains("phase-06"));
                }
                SpecDirectoryEvent::SpecCreated(path) => {
                    assert!(path.to_string_lossy().contains("116-test-spec.md"));
                }
                _ => {}
            }
        }
    }
}