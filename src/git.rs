use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryState {
    pub root: PathBuf,
    pub branch: String,
    pub upstream: Option<Upstream>,
    pub remotes: Vec<String>,
    pub files: Vec<FileChange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Upstream {
    pub remote: String,
    pub branch: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileChange {
    pub path: PathBuf,
    pub original_path: Option<PathBuf>,
    pub index: ChangeCode,
    pub worktree: ChangeCode,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeCode {
    Unchanged,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    UpdatedButUnmerged,
    Untracked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffPreview {
    pub text: String,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitResult {
    pub short_hash: String,
    pub push_output: Option<String>,
}

const MAX_DIFF_BYTES: usize = 48 * 1024;
const MAX_DIFF_LINES: usize = 600;

impl RepositoryState {
    pub fn load(start: &Path) -> Result<Self> {
        let root = git_output(start, ["rev-parse", "--show-toplevel"])?;
        let root = PathBuf::from(root.trim());
        let output = git_output_bytes(&root, ["status", "--porcelain=v2", "--branch", "-z"])?;
        let mut state = parse_status(&root, &output)?;
        state.remotes = git_output(&root, ["remote"])?
            .lines()
            .map(str::trim)
            .filter(|remote| !remote.is_empty())
            .map(ToString::to_string)
            .collect();
        state.upstream = configured_upstream(&root, &state.branch).ok().flatten();
        for index in 0..state.files.len() {
            let file = state.files[index].clone();
            if let Ok((additions, deletions)) = state.file_stats(&file) {
                state.files[index].additions = additions;
                state.files[index].deletions = deletions;
            }
        }
        Ok(state)
    }

    pub fn diff_for(&self, file: &FileChange) -> Result<DiffPreview> {
        let mut sections = Vec::new();
        if file.index != ChangeCode::Unchanged {
            sections.push((
                "STAGED",
                git_output_bytes(
                    &self.root,
                    [
                        "diff",
                        "--cached",
                        "--",
                        file.path.to_string_lossy().as_ref(),
                    ],
                )?,
            ));
        }
        if file.worktree != ChangeCode::Unchanged && file.worktree != ChangeCode::Untracked {
            sections.push((
                "UNSTAGED",
                git_output_bytes(
                    &self.root,
                    ["diff", "--", file.path.to_string_lossy().as_ref()],
                )?,
            ));
        }
        if file.worktree == ChangeCode::Untracked {
            let output = Command::new("git")
                .args([
                    "diff",
                    "--no-index",
                    "--",
                    "/dev/null",
                    file.path.to_string_lossy().as_ref(),
                ])
                .current_dir(&self.root)
                .output()
                .context("could not run Git diff for untracked file")?;
            if !output.status.success() && output.status.code() != Some(1) {
                bail!(
                    "git diff failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
            sections.push(("UNTRACKED", output.stdout));
        }

        let mut text = String::new();
        for (label, section) in sections {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&format!("--- {label} ---\n"));
            text.push_str(&String::from_utf8_lossy(&section));
        }
        if text.trim().is_empty() {
            text = "No textual diff is available for this path.".to_string();
        }
        Ok(truncate_diff(text))
    }

    fn file_stats(&self, file: &FileChange) -> Result<(usize, usize)> {
        let output = git_output(
            &self.root,
            [
                "diff",
                "--numstat",
                "HEAD",
                "--",
                file.path.to_string_lossy().as_ref(),
            ],
        )?;
        if let Some(line) = output.lines().next() {
            let mut fields = line.split_whitespace();
            let additions = fields
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            let deletions = fields
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            return Ok((additions, deletions));
        }
        if file.is_untracked() {
            let additions = fs::read_to_string(self.root.join(&file.path))
                .map(|text| text.lines().count())
                .unwrap_or(0);
            return Ok((additions, 0));
        }
        Ok((0, 0))
    }

    pub fn commit_selected(&self, selected: &[PathBuf], message: &str) -> Result<CommitResult> {
        if selected.is_empty() {
            bail!("Select at least one change before committing.");
        }
        let paths: Vec<_> = selected.iter().map(|path| path.as_os_str()).collect();
        let add = Command::new("git")
            .arg("add")
            .arg("--")
            .args(&paths)
            .current_dir(&self.root)
            .output()
            .context("could not stage selected changes")?;
        if !add.status.success() {
            bail!(
                "git add failed: {}",
                String::from_utf8_lossy(&add.stderr).trim()
            );
        }
        let commit = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.root)
            .output()
            .context("could not create commit")?;
        if !commit.status.success() {
            bail!(
                "git commit failed: {}",
                String::from_utf8_lossy(&commit.stderr).trim()
            );
        }
        let short_hash = git_output(&self.root, ["rev-parse", "--short", "HEAD"])?
            .trim()
            .to_string();
        Ok(CommitResult {
            short_hash,
            push_output: None,
        })
    }

    pub fn push(&self, remote: &str, branch: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["push", remote, &format!("HEAD:{branch}")])
            .current_dir(&self.root)
            .output()
            .context("could not push commit")?;
        if !output.status.success() {
            bail!(
                "git push failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn configured_upstream(root: &Path, branch: &str) -> Result<Option<Upstream>> {
    if branch == "detached HEAD" {
        return Ok(None);
    }
    let remote = match git_output(
        root,
        ["config", "--get", &format!("branch.{branch}.remote")],
    ) {
        Ok(remote) => remote.trim().to_string(),
        Err(_) => return Ok(None),
    };
    let merge = match git_output(root, ["config", "--get", &format!("branch.{branch}.merge")]) {
        Ok(merge) => merge.trim().to_string(),
        Err(_) => return Ok(None),
    };
    let branch = merge
        .strip_prefix("refs/heads/")
        .unwrap_or(&merge)
        .to_string();
    Ok((!remote.is_empty() && !branch.is_empty()).then_some(Upstream { remote, branch }))
}

impl FileChange {
    pub fn is_conflict(&self) -> bool {
        self.index == ChangeCode::UpdatedButUnmerged
            || self.worktree == ChangeCode::UpdatedButUnmerged
    }

    pub fn is_untracked(&self) -> bool {
        self.index == ChangeCode::Untracked || self.worktree == ChangeCode::Untracked
    }

    pub fn is_safe_to_select(&self) -> bool {
        !self.is_conflict()
    }

    pub fn indicator(&self) -> String {
        if self.is_conflict() {
            return "CONFLICT".to_string();
        }
        if self.is_untracked() {
            return "UNTRACKED".to_string();
        }
        let mut labels = Vec::new();
        if self.index != ChangeCode::Unchanged {
            labels.push(format!("STAGED {}", self.index.label()));
        }
        if self.worktree != ChangeCode::Unchanged {
            labels.push(format!("UNSTAGED {}", self.worktree.label()));
        }
        labels.join(" + ")
    }
}

impl ChangeCode {
    fn from_status(value: u8) -> Self {
        match value {
            b'M' => Self::Modified,
            b'A' => Self::Added,
            b'D' => Self::Deleted,
            b'R' => Self::Renamed,
            b'C' => Self::Copied,
            b'U' => Self::UpdatedButUnmerged,
            b'?' => Self::Untracked,
            _ => Self::Unchanged,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Unchanged => "",
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Copied => "C",
            Self::UpdatedButUnmerged => "U",
            Self::Untracked => "?",
        }
    }
}

fn git_output<const N: usize>(cwd: &Path, args: [&str; N]) -> Result<String> {
    let output = git_output_bytes(cwd, args)?;
    Ok(String::from_utf8_lossy(&output).into_owned())
}

fn git_output_bytes<const N: usize>(cwd: &Path, args: [&str; N]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("could not run git in {}", cwd.display()))?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(output.stdout)
}

pub fn parse_status(root: &Path, output: &[u8]) -> Result<RepositoryState> {
    let mut branch = "detached HEAD".to_string();
    let mut files = Vec::new();
    let mut records = output.split(|byte| *byte == 0).peekable();
    while let Some(record) = records.next() {
        if record.is_empty() {
            continue;
        }
        if record.starts_with(b"# branch.head ") {
            let value = String::from_utf8_lossy(&record[14..]);
            branch = if value == "(detached)" {
                "detached HEAD".to_string()
            } else {
                value.into_owned()
            };
            continue;
        }
        match record.first() {
            Some(b'1') => files.push(parse_ordinary(record)?),
            Some(b'2') => {
                let mut file = parse_rename(record)?;
                file.original_path = records.next().map(path_from_bytes);
                files.push(file);
            }
            Some(b'u') => files.push(parse_unmerged(record)?),
            Some(b'?') => files.push(FileChange {
                path: path_from_bytes(bytes_after_prefix(record, b"? ")?),
                original_path: None,
                index: ChangeCode::Untracked,
                worktree: ChangeCode::Untracked,
                additions: 0,
                deletions: 0,
            }),
            _ => {}
        }
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(RepositoryState {
        root: root.to_path_buf(),
        branch,
        upstream: None,
        remotes: Vec::new(),
        files,
    })
}

fn parse_ordinary(record: &[u8]) -> Result<FileChange> {
    let parts = split_fields(record, 9)?;
    status_file(parts[1], parts[8], None)
}

fn parse_rename(record: &[u8]) -> Result<FileChange> {
    let parts = split_fields(record, 10)?;
    status_file(parts[1], parts[9], None)
}

fn parse_unmerged(record: &[u8]) -> Result<FileChange> {
    let parts = split_fields(record, 11)?;
    status_file(parts[1], parts[10], None)
}

fn split_fields(record: &[u8], expected: usize) -> Result<Vec<&[u8]>> {
    let parts: Vec<_> = record.splitn(expected, |byte| *byte == b' ').collect();
    if parts.len() != expected {
        bail!(
            "unexpected git status record: {}",
            String::from_utf8_lossy(record)
        );
    }
    Ok(parts)
}

fn status_file(status: &[u8], path: &[u8], original_path: Option<PathBuf>) -> Result<FileChange> {
    if status.len() != 2 {
        bail!(
            "unexpected git status code: {}",
            String::from_utf8_lossy(status)
        );
    }
    Ok(FileChange {
        path: path_from_bytes(path),
        original_path,
        index: ChangeCode::from_status(status[0]),
        worktree: ChangeCode::from_status(status[1]),
        additions: 0,
        deletions: 0,
    })
}

fn bytes_after_prefix<'a>(record: &'a [u8], prefix: &[u8]) -> Result<&'a [u8]> {
    record
        .strip_prefix(prefix)
        .context("unexpected git status path record")
}

fn path_from_bytes(path: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(path).into_owned())
}

fn truncate_diff(text: String) -> DiffPreview {
    let mut truncated = text.len() > MAX_DIFF_BYTES;
    let mut limited = if truncated {
        text[..text.floor_char_boundary(MAX_DIFF_BYTES)].to_string()
    } else {
        text
    };
    let lines: Vec<_> = limited.lines().collect();
    if lines.len() > MAX_DIFF_LINES {
        limited = lines[..MAX_DIFF_LINES].join("\n");
        truncated = true;
    }
    if truncated {
        limited.push_str("\n\n[diff preview limited to 48 KiB / 600 lines]");
    }
    DiffPreview {
        text: limited,
        truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mixed_status_and_rename() {
        let status = b"# branch.head main\0# branch.oid abc\0"
            .iter()
            .chain(b"1 MM N... 100644 100644 100644 a b src/a.rs\0".iter())
            .chain(b"2 R. N... 100644 100644 100644 a b R100 new.rs\0old.rs\0".iter())
            .chain(b"? untracked.txt\0".iter())
            .copied()
            .collect::<Vec<_>>();
        let state = parse_status(Path::new("/repo"), &status).unwrap();
        assert_eq!(state.branch, "main");
        assert_eq!(state.upstream, None);
        assert!(state.remotes.is_empty());
        assert_eq!(state.files.len(), 3);
        let modified = state
            .files
            .iter()
            .find(|file| file.path == Path::new("src/a.rs"))
            .unwrap();
        assert_eq!(modified.indicator(), "STAGED M + UNSTAGED M");
        let renamed = state
            .files
            .iter()
            .find(|file| file.path == Path::new("new.rs"))
            .unwrap();
        assert_eq!(renamed.original_path.as_deref(), Some(Path::new("old.rs")));
    }

    #[test]
    fn parses_conflicts_and_detached_head() {
        let state = parse_status(Path::new("/repo"), b"# branch.head (detached)\0u UU N... 100644 100644 100644 100644 a b c conflicted.rs\0").unwrap();
        assert_eq!(state.branch, "detached HEAD");
        assert!(state.files[0].is_conflict());
        assert_eq!(state.files[0].indicator(), "CONFLICT");
    }

    #[test]
    fn limits_large_diffs_at_a_character_boundary() {
        let preview = truncate_diff("x".repeat(MAX_DIFF_BYTES + 10));
        assert!(preview.truncated);
        assert!(preview.text.contains("diff preview limited"));
    }
}
