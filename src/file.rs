use crate::{Branch, Commit, Error};
use std::path::PathBuf;

/// A reference to a file on a certain commit.
pub struct File<'a> {
    pub(crate) branch: &'a Branch<'a>,
    pub(crate) path: &'a str,
    pub(crate) entry: MaybeOwned<'a, git2::TreeEntry<'a>>,
}

pub(crate) enum MaybeOwned<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> std::ops::Deref for MaybeOwned<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(b) => b,
        }
    }
}

impl<'a, T> From<&'a T> for MaybeOwned<'a, T> {
    fn from(b: &'a T) -> Self {
        Self::Borrowed(b)
    }
}
impl<'a, T> From<T> for MaybeOwned<'a, T> {
    fn from(b: T) -> Self {
        Self::Owned(b)
    }
}

impl<'a> File<'a> {
    /// The path of the file in the repository.
    pub fn path(&self) -> String {
        format!("{}{}", self.path, self.entry.name().unwrap())
    }

    /// Returns `true` if the selected file is an actual file. See also [File::is_dir].
    pub fn is_file(&self) -> bool {
        self.entry.kind() == Some(git2::ObjectType::Blob)
    }

    /// Returns `true` if the selected file is a directory. See also [File::is_file].
    pub fn is_dir(&self) -> bool {
        self.entry.kind() == Some(git2::ObjectType::Tree)
    }

    /// Read the raw contents of the file.
    pub fn read_content(&self) -> Result<Vec<u8>, Error> {
        let object = self.entry.to_object(self.branch.repo)?;
        let blob = object.peel_to_blob()?;
        Ok(blob.content().to_vec())
    }

    /// Read the contents of the file as a string.
    /// Will use a lossy encoding.
    /// See [String::from_utf8_lossy] for more information.
    pub fn read_content_string(&self) -> Result<String, Error> {
        let object = self.entry.to_object(self.branch.repo)?;
        let blob = object.peel_to_blob()?;
        let content = String::from_utf8_lossy(blob.content());
        Ok(content.to_string())
    }

    /// Iterate over the commits that modified this file. The newest commit is listed first.
    pub fn history(&'a self) -> Result<impl Iterator<Item = Result<Commit<'a>, Error>>, Error> {
        FileHistory::new(self)
    }
}

#[test]
fn test_file_is_dir() {
    use crate::*;
    let repo = Repo::open(".").unwrap();
    let branch = match repo.current_branch() {
        Ok(Some(branch)) => branch,
        _ => {
            // in CI we don't always have a HEAD branch.
            return;
        }
    };
    branch
        .files(|f| {
            if f.path() == "src" || f.path() == ".github" || f.path() == ".github/workflows" {
                assert!(f.is_dir());
            } else {
                assert!(!f.is_dir());
            }
            Ok(())
        })
        .unwrap();
}

// Huge thanks to kvzn on github
// https://github.com/rust-lang/git2-rs/issues/588#issuecomment-658510497

struct FileHistory<'a> {
    file: &'a File<'a>,
    revwalk: git2::Revwalk<'a>,
    file_path: PathBuf,
}

impl<'a> FileHistory<'a> {
    fn new(file: &'a File<'a>) -> Result<Self, Error> {
        let commit = file.branch.branch.get().peel_to_commit()?;

        let mut revwalk = file.branch.repo.revwalk()?;
        revwalk.push(commit.id())?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let file_path = PathBuf::from(file.path());

        Ok(Self {
            file,
            revwalk,
            file_path,
        })
    }

    fn get_commit_from_oid(
        &mut self,
        oid: Result<git2::Oid, Error>,
    ) -> Result<Option<git2::Commit<'a>>, Error> {
        let oid = oid?;
        let commit = self.file.branch.repo.find_commit(oid)?;
        let tree = commit.tree()?;
        let old_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };
        let diff = self
            .file
            .branch
            .repo
            .diff_tree_to_tree(old_tree.as_ref(), Some(&tree), None)?;
        let mut deltas = diff.deltas();

        let contains = deltas.any(|dd| {
            let new_file_path = dd.new_file().path().unwrap();
            new_file_path.eq(&self.file_path)
        });

        Ok(if contains { Some(commit) } else { None })
    }
}

impl<'a> Iterator for FileHistory<'a> {
    type Item = Result<Commit<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(oid) = self.revwalk.next() {
            match self.get_commit_from_oid(oid) {
                Err(e) => return Some(Err(e)),
                Ok(None) => continue,
                Ok(Some(commit)) => return Some(Ok(Commit::from_file(self.file, commit))),
            }
        }

        None
    }
}
