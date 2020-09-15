//! Braindead git sync:
//!   git add .
//!   git commit -am update
//!   git pull origin master
//!   git push origin master

use std::{fs, io, path::Path, thread, time};

/// How many seconds to wait before syncing.
const SYNC_WAIT: u64 = 30;

/// Start the syncing service.
pub fn start(root: &str) -> Result<(), io::Error> {
    if !is_git_repo(root) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a git repo", root),
        ));
    }

    println!("~> running sync service");
    let root = root.to_string();
    thread::spawn(move || sync_periodically(&root));
    Ok(())
}

/// Is this wiki a git repo?
fn is_git_repo(root: &str) -> bool {
    let dir = format!("{}.git", root);
    let path = Path::new(&dir);
    if let Ok(file) = fs::File::open(path) {
        if let Ok(meta) = file.metadata() {
            return meta.is_dir();
        }
    }
    false
}

/// Run the sync and then sleep.
fn sync_periodically(root: &str) -> Result<(), io::Error> {
    // set current dir to wiki
    let period = time::Duration::from_millis(SYNC_WAIT * 1000);
    loop {
        save_changes(root)?;
        sync_changes(root)?;
        thread::sleep(period);
    }
}

macro_rules! git {
    ($root:expr, $($arg:expr),+) => {
        git($root, &[$($arg),+])
    };
}

/// Try to add and commit any new or modified wiki pages.
fn save_changes(root: &str) -> Result<bool, io::Error> {
    let pending = git!(root, "status", "-s")?;
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
    git!(root, "add", ".")?;
    git!(root, "commit", "-am", &status)?;
    Ok(true)
}

fn sync_changes(root: &str) -> Result<bool, io::Error> {
    println!("~> syncing changes");
    git!(root, "pull", "origin", "master")?;
    git!(root, "push", "origin", "master")?;
    Ok(true)
}

fn git(root: &str, args: &[&str]) -> Result<String, std::io::Error> {
    shell!(
        "git --git-dir {root}.git --work-tree {root} {args}",
        root = root,
        args = args.join(" ")
    )
}
