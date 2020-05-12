//! Braindead git sync:
//!   git add .
//!   git commit -am update
//!   git pull origin master
//!   git push origin master

use {
    crate::{shell, wiki_root},
    std::{fs, io, path::Path, thread, time},
};

/// How many seconds to wait before syncing.
const SYNC_WAIT: u64 = 30;

/// Start the syncing service.
pub fn start() -> Result<(), io::Error> {
    if !is_git_repo() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "wiki is not a git repo",
        ));
    }

    println!("~> running sync service");
    thread::spawn(sync_periodically);
    Ok(())
}

/// Is this wiki a git repo?
fn is_git_repo() -> bool {
    let path = format!("{}/.git", wiki_root());
    let path = Path::new(&path);
    if let Ok(file) = fs::File::open(path) {
        if let Ok(meta) = file.metadata() {
            return meta.is_dir();
        }
    }
    false
}

/// Run the sync and then sleep.
fn sync_periodically() -> Result<(), io::Error> {
    let period = time::Duration::from_millis(SYNC_WAIT * 1000);
    loop {
        save_changes()?;
        sync_changes()?;
        thread::sleep(period);
    }
}

/// Try to add and commit any new or modified wiki pages.
fn save_changes() -> Result<bool, io::Error> {
    let pending = shell("git", &["status", "-s"])?;
    if pending.is_empty() {
        return Ok(false);
    }

    // get a list of files that changed
    let changes = pending
        .split('\n')
        .map(|file| if file.len() > 3 { &file[3..] } else { file }.trim())
        .filter(|f| !f.is_empty())
        .collect::<Vec<_>>();
    let status = changes.join(if changes.len() == 1 { "" } else { ", " });

    println!("~> saving changes: {}", status);
    shell("git", &["add", "."])?;
    shell("git", &["commit", "-am", &status])?;
    Ok(true)
}

fn sync_changes() -> Result<bool, io::Error> {
    println!("~> syncing changes");
    shell("git", &["pull", "origin", "master"])?;
    shell("git", &["push", "origin", "master"])?;
    Ok(true)
}
