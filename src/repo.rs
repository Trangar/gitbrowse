use crate::{Branch, Error, NameWithId};
use std::path::Path;

/// A reference to an opened repository.
pub struct Repo {
    repo: git2::Repository,
}

impl Repo {
    /// Open the repository at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Repo, Error> {
        let repo = git2::Repository::open(path)?;
        Ok(Repo { repo })
    }

    /// Browse a branch with the given name. Will return an error if the branch is not found.
    pub fn browse_branch<'a>(&'a self, branch: &str) -> Result<Branch<'a>, Error> {
        Branch::new(&self.repo, branch)
    }

    /// Attempt to get the current branch. This will return `None` if the current git `HEAD` is detached.
    pub fn current_branch(&self) -> Result<Option<Branch>, Error> {
        let head = self.repo.head()?;
        Ok(if let Some(target) = head.symbolic_target() {
            Some(self.browse_branch(target)?)
        } else {
            Some(Branch::new_by_reference(&self.repo, head)?)
        })
    }

    /// List all branches and the ID of the last commit of that branch.
    pub fn list_branches_with_newest_commit_id(&self) -> Result<Vec<NameWithId>, Error> {
        let branches = self.repo.branches(Some(git2::BranchType::Local))?;

        let mut result = Vec::new();
        for branch in branches {
            let branch = branch?.0;
            if let (Some(name), Some(oid)) = (branch.name()?, branch.get().target()) {
                result.push(NameWithId {
                    commit_id: oid.to_string(),
                    name: name.to_owned(),
                });
            }
        }
        Ok(result)
    }

    /// List all branches.
    pub fn list_branches(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .list_branches_with_newest_commit_id()?
            .into_iter()
            .map(|n| n.name)
            .collect())
    }

    /// List all tags and the commit ID of that tag.
    pub fn list_tags_with_commit_id(&self) -> Result<Vec<NameWithId>, Error> {
        let mut result = Vec::new();

        self.repo.tag_foreach(|oid, bytes| {
            if let Ok(mut name) = std::str::from_utf8(bytes) {
                if let Some(index) = name.rfind('/') {
                    name = &name[index + 1..];
                }
                result.push(NameWithId {
                    commit_id: oid.to_string(),
                    name: name.to_owned(),
                });
            }
            true
        })?;
        Ok(result)
    }

    /// List all tags.
    pub fn list_tags(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .list_tags_with_commit_id()?
            .into_iter()
            .map(|n| n.name)
            .collect())
    }
}
