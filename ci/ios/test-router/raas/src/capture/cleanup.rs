use std::path::Path;
use tokio::{fs, time::Duration};
use uuid::Uuid;

const ONE_DAY_AGO: Duration = Duration::from_secs(24 * 60 * 60); // 86400 seconds in a day

pub async fn delete_old_captures() -> std::io::Result<()> {
    delete_old_captures_inner(&super::Capture::capture_dir_path()).await
}

async fn delete_old_captures_inner(dir: &Path) -> std::io::Result<()> {
    let mut entries = fs::read_dir(dir).await?;

    // Collecting tasks to join later
    let mut delete_tasks = vec![];

    // Iterate over directory entries
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_file() && should_delete_capture_file(&path).await {
            // Spawn a task to delete the file
            let path = path.clone();
            delete_tasks.push(tokio::spawn(async move {
                if let Err(e) = fs::remove_file(&path).await {
                    eprintln!("Failed to delete {:?}: {}", path, e);
                }
            }));
        }
    }

    // Wait for all delete tasks to complete
    for task in delete_tasks {
        let _ = task.await;
    }

    Ok(())
}

//
async fn should_delete_capture_file(path: &Path) -> bool {
    // Check if the file name is a valid UUID
    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        if Uuid::parse_str(file_name).is_ok() {
            // Check the file's metadata
            let Some(metadata) = fs::metadata(&path).await.ok() else {
                return false;
            };
            if let Ok(modified_time) = metadata.modified() {
                // Calculate the elapsed time since the file was modified
                if let Ok(duration) = modified_time.elapsed() {
                    return duration > ONE_DAY_AGO;
                }
            }
        }
    }
    false
}
