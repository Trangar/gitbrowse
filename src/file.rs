use crate::{Branch, Commit, Error};
use std::path::Path;

/// A reference to a file on a certain commit.
pub struct File<'a> {
    pub(crate) branch: &'a Branch<'a>,
    pub(crate) entry: git2::TreeEntry<'a>,
}

impl<'a> File<'a> {
    /// The path of the file in the repository.
    pub fn path(&self) -> &str {
        self.entry.name().unwrap()
    }

    /// Read the contents of the file as a string.
    /// Will use a lossy encoding.
    /// See [String::from_utf_lossy] for more information.
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

// Huge thanks to kvzn on github
// https://github.com/rust-lang/git2-rs/issues/588#issuecomment-658510497

struct FileHistory<'a> {
    file: &'a File<'a>,
    revwalk: git2::Revwalk<'a>,
    file_path: &'a Path,
}

impl<'a> FileHistory<'a> {
    fn new(file: &'a File<'a>) -> Result<Self, Error> {
        let commit = file.branch.branch.get().peel_to_commit()?;

        let mut revwalk = file.branch.repo.revwalk()?;
        revwalk.push(commit.id())?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let file_path = std::path::Path::new(file.path());

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
            new_file_path.eq(self.file_path)
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
