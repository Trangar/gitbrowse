use crate::{Commit, Error, File};

/// A reference to a branch on the git repo.
pub struct Branch<'a> {
    pub(crate) repo: &'a git2::Repository,
    pub(crate) branch: git2::Branch<'a>,
    pub(crate) tree: git2::Tree<'a>,
}

impl<'a> Branch<'a> {
    pub(crate) fn new(repo: &'a git2::Repository, branch: &str) -> Result<Self, Error> {
        let branch = repo.find_branch(branch, git2::BranchType::Local)?;
        let tree = branch.get().peel_to_tree()?;
        Ok(Branch { repo, branch, tree })
    }

    pub(crate) fn new_by_reference(
        repo: &'a git2::Repository,
        reference: git2::Reference<'a>,
    ) -> Result<Self, Error> {
        let branch = git2::Branch::wrap(reference);
        let tree = branch.get().peel_to_tree()?;
        Ok(Branch { repo, branch, tree })
    }

    /// Get the name of the current branch
    pub fn name(&self) -> &str {
        self.branch.name().unwrap().unwrap()
    }

    /// List all the files on the newest commit of the current branch.
    pub fn files(&'a self, mut cb: impl FnMut(File<'_>) -> Result<(), Error>) -> Result<(), Error> {
        let mut result = Ok(());
        self.tree
            .walk(git2::TreeWalkMode::PreOrder, |path, entry| {
                result = cb(File {
                    branch: self,
                    path,
                    entry: entry.into(),
                });
                if result.is_err() {
                    git2::TreeWalkResult::Abort
                } else {
                    git2::TreeWalkResult::Ok
                }
            })?;
        result
    }

    /// Returns an iterator of all the commits on this branch. Will return the newest commit first.
    pub fn commits(&'a self) -> Result<impl Iterator<Item = Commit<'a>>, Error> {
        let commit = self.branch.get().peel_to_commit()?;
        Ok(BranchCommitIterator {
            commit: Some(commit),
        })
    }

    /// Get a file by the given path in the last commit of the current branch.
    ///
    /// Will return `Ok(None)` if the file is not found.
    pub fn get_file_by_path(&'a self, path: &'a str) -> Result<File<'a>, Error> {
        // HACK, this needs to be improved
        let ancestor = if let Some(idx) = path.rfind('/') {
            &path[..idx + 1]
        } else {
            ""
        };
        self.tree.get_path(std::path::Path::new(path)).map(|entry| File {
            branch: self,
            path: ancestor,
            entry: entry.into()
        })
    }
}

/*
TODO: Can't figure out the lifetimes for this

struct BranchFileIterator<'a, 'b> {
    branch: &'b Branch<'a>,
    trees: Vec<(git2::Tree<'a>, usize)>,
}

impl<'a, 'b> BranchFileIterator<'a, 'b> {
    pub(crate) fn new(branch: &'b Branch<'a>) -> Self {
        Self {
            trees: vec![(branch.tree.clone(), 0)],
            branch,
        }
    }
}

impl<'a, 'b> Iterator for BranchFileIterator<'a, 'b> where 'b: 'a {
    type Item = File<'a>;

    fn next(&mut self) -> Option<File<'a>> {
        loop {
            let (last_tree, last_tree_idx): (git2::Tree<'a>, &mut usize) = match self.trees.last_mut() {
                Some((last_tree, last_tree_idx)) => (last_tree.clone(), last_tree_idx),
                None => return None
            };

            let file: git2::TreeEntry<'a> = match last_tree.get(*last_tree_idx) {
                Some(f) => f.clone(),
                None => {
                    self.trees.pop();
                    continue;
                }
            };
            *last_tree_idx += 1;

            match file.kind() {
                Some(git2::ObjectType::Blob) => {
                    return Some(File {
                        branch: self.branch,
                        entry: file.into(),
                    })
                }
                Some(git2::ObjectType::Tree) => {
                    let tree: git2::Tree<'a> = file.to_object(&self.branch.repo).ok()?.peel_to_tree().ok()?.clone();
                    self.trees.push((tree, tree.len()));
                }
                x => {
                    panic!("Unknown file kind: {:?}", x);
                }
            }
        }
    }
}
*/

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
fn test_files() {
    let repo = crate::Repo::open(".").unwrap();

    // Sometimes in CI we don't have any branches
    let branch = match repo.current_branch().unwrap() {
        Some(branch) => branch,
        None => {
            eprintln!("Warning: No branch found");
            return;
        }
    };

    println!("Found branch {:?}", branch.name());

    let mut files = Vec::new();
    branch
        .files(|file| {
            files.push(file.path());
            Ok(())
        })
        .unwrap();
    println!("Found {} files", files.len());
    println!("{:#?}", files);

    assert!(files.iter().any(|f| f.starts_with("src/")));
    assert!(files.iter().any(|f| f == "Cargo.toml"));

    let file = branch.get_file_by_path("src/lib.rs").unwrap();
    assert_eq!("src/lib.rs", file.path());
}

#[test]
fn test_commits() {
    let repo = crate::Repo::open(".").unwrap();

    // Sometimes in CI we don't have any branches
    let branch = match repo.current_branch().unwrap() {
        Some(branch) => branch,
        None => {
            eprintln!("Warning: No branch found");
            return;
        }
    };

    println!("Found branch {:?}", branch.name());

    let commits: Vec<_> = branch.commits().unwrap().collect();
    assert!(!commits.is_empty());

    println!("Branch has {} commits", commits.len());

    // on CI this checkout only has 1 commit, so protect against that
    if commits.len() > 1 {
        // First commit of this repo
        assert_eq!("Initial commit", commits.last().unwrap().message());
        assert_eq!("4c247b6", commits.last().unwrap().id());
    }
}
