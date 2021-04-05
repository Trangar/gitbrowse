use crate::File;

/// A reference to a branch on the git repo.
pub struct Branch<'a> {
    pub(crate) repo: &'a git2::Repository,
    pub(crate) branch: git2::Branch<'a>,
    pub(crate) tree: git2::Tree<'a>,
}

impl<'a> Branch<'a> {
    pub(crate) fn new(repo: &'a git2::Repository, branch: &str) -> Result<Self, git2::Error> {
        let branch = repo.find_branch(branch, git2::BranchType::Local)?;
        let tree = branch.get().peel_to_tree()?;
        Ok(Branch { repo, branch, tree })
    }

    /// Get the name of the current branch
    pub fn name(&self) -> &str {
        self.branch.name().unwrap().unwrap()
    }

    /// List all the files on the newest commit of the current branch.
    pub fn list_files(&'a self) -> impl Iterator<Item = File<'a>> {
        let iter = self.tree.iter();
        BranchFileIterator { branch: self, iter }
    }

    /// Get a file by the given path in the last commit of the current branch.
    ///
    /// Will return `Ok(None)` if the file is not found.
    pub fn get_file_by_path(&'a self, path: &'a str) -> Result<Option<File<'a>>, git2::Error> {
        let entry = match self.tree.get_name(path) {
            Some(e) => e,
            None => return Ok(None),
        };

        Ok(Some(File {
            branch: self,
            entry,
        }))
    }
}

struct BranchFileIterator<'a> {
    branch: &'a Branch<'a>,
    iter: git2::TreeIter<'a>,
}

impl<'a> Iterator for BranchFileIterator<'a> {
    type Item = File<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let file = self.iter.next()?;

            if file.kind() == Some(git2::ObjectType::Blob) {
                return Some(File {
                    branch: self.branch,
                    entry: file,
                });
            }
        }
    }
}
