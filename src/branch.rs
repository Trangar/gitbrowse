use crate::{Commit, File};

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
    pub fn files(&'a self) -> impl Iterator<Item = File<'a>> {
        let iter = self.tree.iter();
        BranchFileIterator { branch: self, iter }
    }

    /// Returns an iterator of all the commits on this branch. Will return the newest commit first.
    pub fn commits(&'a self) -> Result<impl Iterator<Item = Commit<'a>>, git2::Error> {
        let commit = self.branch.get().peel_to_commit()?;
        Ok(BranchCommitIterator {
            commit: Some(commit),
        })
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

struct BranchCommitIterator<'a> {
    commit: Option<git2::Commit<'a>>,
}

impl<'a> Iterator for BranchCommitIterator<'a> {
    type Item = Commit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let commit = self.commit.take()?;

        if commit.parent_count() > 0 {
            self.commit = commit.parent(0).ok();
        }
        Some(Commit { commit })
    }
}

#[test]
fn test_commits() {
    let repo = crate::Repo::open(".").unwrap();
    let branches = repo.list_branches().unwrap();
    let branch_name = branches.first().unwrap();
    let branch = repo.browse_branch(&branch_name).unwrap();

    let commits: Vec<_> = branch.commits().unwrap().collect();
    assert!(!commits.is_empty());

    // First commit of this repo
    assert_eq!("Initial commit", commits.last().unwrap().message());
    assert_eq!("4c247b6", commits.last().unwrap().id());
}
