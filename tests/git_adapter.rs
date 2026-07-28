use fgit::git::{ChangeCode, RepositoryState};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn repository() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path();
    git(repo, &["init", "-b", "main"]);
    git(repo, &["config", "user.name", "FGIT test"]);
    git(repo, &["config", "user.email", "fgit@example.test"]);
    fs::write(repo.join("mixed.txt"), "base\n").unwrap();
    fs::write(repo.join("deleted.txt"), "delete me\n").unwrap();
    fs::write(repo.join("old-name.txt"), "rename me\n").unwrap();
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", "baseline"]);
    temp
}

#[test]
fn reads_clean_and_all_changed_path_categories() {
    let temp = repository();
    let repo = temp.path();
    assert!(RepositoryState::load(repo).unwrap().files.is_empty());

    fs::write(repo.join("mixed.txt"), "staged\n").unwrap();
    git(repo, &["add", "mixed.txt"]);
    fs::write(repo.join("mixed.txt"), "staged\nunstaged\n").unwrap();
    fs::remove_file(repo.join("deleted.txt")).unwrap();
    fs::rename(repo.join("old-name.txt"), repo.join("new-name.txt")).unwrap();
    git(repo, &["add", "-A", "old-name.txt", "new-name.txt"]);
    fs::write(repo.join("untracked.txt"), "new\n").unwrap();

    let state = RepositoryState::load(repo).unwrap();
    let mixed = state
        .files
        .iter()
        .find(|file| file.path == Path::new("mixed.txt"))
        .unwrap();
    assert_eq!(mixed.index, ChangeCode::Modified);
    assert_eq!(mixed.worktree, ChangeCode::Modified);
    assert!(
        state
            .files
            .iter()
            .any(|file| file.worktree == ChangeCode::Deleted)
    );
    let renamed = state
        .files
        .iter()
        .find(|file| file.path == Path::new("new-name.txt"))
        .unwrap();
    assert_eq!(renamed.index, ChangeCode::Renamed);
    assert_eq!(
        renamed.original_path.as_deref(),
        Some(Path::new("old-name.txt"))
    );
    assert!(state.files.iter().any(|file| file.is_untracked()));
}

#[test]
fn reads_conflicts_without_modifying_the_repository() {
    let temp = repository();
    let repo = temp.path();
    git(repo, &["checkout", "-b", "topic"]);
    fs::write(repo.join("mixed.txt"), "topic\n").unwrap();
    git(repo, &["commit", "-am", "topic edit"]);
    git(repo, &["checkout", "main"]);
    fs::write(repo.join("mixed.txt"), "main\n").unwrap();
    git(repo, &["commit", "-am", "main edit"]);
    let merge = Command::new("git")
        .args(["merge", "topic"])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(!merge.status.success());

    let state = RepositoryState::load(repo).unwrap();
    assert!(state.files.iter().any(|file| file.is_conflict()));
    assert!(repo.join("mixed.txt").exists());
}

#[test]
fn lazily_limits_a_large_untracked_diff() {
    let temp = repository();
    let repo = temp.path();
    fs::write(repo.join("large.txt"), "x\n".repeat(40_000)).unwrap();
    let state = RepositoryState::load(repo).unwrap();
    let file = state
        .files
        .iter()
        .find(|file| file.path == Path::new("large.txt"))
        .unwrap();
    let diff = state.diff_for(file).unwrap();
    assert!(diff.truncated);
    assert!(diff.text.contains("diff preview limited"));
}
