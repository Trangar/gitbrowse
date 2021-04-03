use crate::{Branch, Commit};

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
    pub fn read_content_string(&self) -> Result<String, git2::Error> {
        let object = self.entry.to_object(self.branch.repo)?;
        let blob = object.peel_to_blob()?;
        let content = String::from_utf8_lossy(blob.content());
        Ok(content.to_string())
    }

    /// Iterate over the commits that modified this file. The newest commit is listed first.
    pub fn history(&'a self, mut cb: impl FnMut(Commit<'a>)) -> Result<(), git2::Error> {
        // Huge thanks to kvzn on github
        // https://github.com/rust-lang/git2-rs/issues/588#issuecomment-658510497

        let commit = self.branch.branch.get().peel_to_commit()?;

        let mut revwalk = self.branch.repo.revwalk()?;
        revwalk.push(commit.id())?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let file_path = std::path::Path::new(self.path());

        for oid in revwalk {
            let oid = oid?;
            let commit = self.branch.repo.find_commit(oid)?;
            let tree = commit.tree()?;

            let old_tree = if commit.parent_count() > 0 {
                Some(commit.parent(0)?.tree()?)
            } else {
                None
            };

            let diff = self
                .branch
                .repo
                .diff_tree_to_tree(old_tree.as_ref(), Some(&tree), None)?;

            let mut deltas = diff.deltas();

            let contains = deltas.any(|dd| {
                let new_file_path = dd.new_file().path().unwrap();
                new_file_path.eq(file_path)
            });

            if contains {
                cb(Commit::from_file(self, commit));
            }
        }

        Ok(())
    }
}
