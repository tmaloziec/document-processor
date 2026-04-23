use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

#[allow(dead_code)]
pub struct FolderWatcher {
    _watcher: notify::RecommendedWatcher,
}

impl FolderWatcher {
    #[allow(dead_code)]
    pub fn new<F>(path: &Path, callback: F) -> Result<Self, notify::Error>
    where
        F: Fn(String) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();

        let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if let Err(e) = tx.send(event) {
                    eprintln!("Error sending event: {}", e);
                }
            }
        })?;

        watcher.watch(path, RecursiveMode::NonRecursive)?;

        // Spawn thread to handle events
        thread::spawn(move || {
            for event in rx {
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    for path in event.paths {
                        if let Some(ext) = path.extension() {
                            let ext = ext.to_string_lossy().to_lowercase();
                            if ["pdf", "docx", "doc", "txt"].contains(&ext.as_str()) {
                                callback(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }
}
