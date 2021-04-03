use crate::{Branch, NameWithId};
use std::path::Path;

/// A reference to an opened repository.
pub struct Repo {
    repo: git2::Repository,
}

impl Repo {
    /// Open the repository at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Repo, git2::Error> {
        let repo = git2::Repository::open(path)?;
        Ok(Repo { repo })
    }

    /// Browse a branch with the given name. Will return an error if the branch is not found.
    pub fn browse_branch<'a>(&'a self, branch: &'a str) -> Result<Branch<'a>, git2::Error> {
        Branch::new(&self.repo, branch)
    }

    /// List all branches and the ID of the last commit of that branch.
    pub fn list_branches_with_newest_commit_id(&self) -> Result<Vec<NameWithId>, git2::Error> {
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
    pub fn list_branches(&self) -> Result<Vec<String>, git2::Error> {
        Ok(self
            .list_branches_with_newest_commit_id()?
            .into_iter()
            .map(|n| n.name)
            .collect())
    }

    /// List all tags and the commit ID of that tag.
    pub fn list_tags_with_commit_id(&self) -> Result<Vec<NameWithId>, git2::Error> {
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
    pub fn list_tags(&self) -> Result<Vec<String>, git2::Error> {
        Ok(self
            .list_tags_with_commit_id()?
            .into_iter()
            .map(|n| n.name)
            .collect())
    }
}
