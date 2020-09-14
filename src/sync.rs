//! Braindead git sync:
//!   git add .
//!   git commit -am update
//!   git pull origin master
//!   git push origin master

use {
    crate::wiki_root,
    std::{fs, io, path::Path, thread, time},
};

/// How many seconds to wait before syncing.
const SYNC_WAIT: u64 = 30;

/// Start the syncing service.
pub fn start() -> Result<(), io::Error> {
    if !is_git_repo() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a git repo", wiki_root()),
        ));
    }

    println!("~> running sync service");
    thread::spawn(sync_periodically);
    Ok(())
}

/// Is this wiki a git repo?
fn is_git_repo() -> bool {
    let dir = format!("{}.git", wiki_root());
    let path = Path::new(&dir);
    if let Ok(file) = fs::File::open(path) {
        if let Ok(meta) = file.metadata() {
            return meta.is_dir();
        }
    }
    false
}

/// Run the sync and then sleep.
fn sync_periodically() -> Result<(), io::Error> {
    // set current dir to wiki
    let period = time::Duration::from_millis(SYNC_WAIT * 1000);
    loop {
        save_changes()?;
        sync_changes()?;
        thread::sleep(period);
    }
}

macro_rules! git {
    ($($arg:expr),+) => {
        git(&[$($arg),+])
    };
}

/// Try to add and commit any new or modified wiki pages.
fn save_changes() -> Result<bool, io::Error> {
    let pending = git!("status", "-s")?;
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
    git!("add", ".")?;
    git!("commit", "-am", &status)?;
    Ok(true)
}

fn sync_changes() -> Result<bool, io::Error> {
    println!("~> syncing changes");
    git!("pull", "origin", "master")?;
    git!("push", "origin", "master")?;
    Ok(true)
}

fn git(args: &[&str]) -> Result<String, std::io::Error> {
    shell!(
        "git --git-dir {root}.git --work-tree {root} {args}",
        root = wiki_root(),
        args = args.join(" ")
    )
}
