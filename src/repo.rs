use crate::Branch;
use std::path::Path;

pub struct Repo {
    repo: git2::Repository,
}

impl Repo {
    pub fn open(path: impl AsRef<Path>) -> Result<Repo, git2::Error> {
        let repo = git2::Repository::open(path)?;
        Ok(Repo { repo })
    }

    pub fn browse_branch<'a>(&'a self, branch: &'a str) -> Result<Branch<'a>, git2::Error> {
        Branch::new(&self.repo, branch)
    }

    pub fn list_branches_with_refs(&self) -> Result<Vec<NameWithRef>, git2::Error> {
        let branches = self.repo.branches(Some(git2::BranchType::Local))?;

        let mut result = Vec::new();
        for branch in branches {
            let branch = branch?.0;
            if let (Some(name), Some(oid)) = (branch.name()?, branch.get().target()) {
                result.push(NameWithRef {
                    r#ref: oid.to_string(),
                    name: name.to_owned(),
                });
            }
        }
        Ok(result)
    }
    pub fn list_branches(&self) -> Result<Vec<String>, git2::Error> {
        Ok(self
            .list_branches_with_refs()?
            .into_iter()
            .map(|n| n.name)
            .collect())
    }

    pub fn list_tags_with_refs(&self) -> Result<Vec<NameWithRef>, git2::Error> {
        let mut result = Vec::new();

        self.repo.tag_foreach(|oid, bytes| {
            if let Ok(mut name) = std::str::from_utf8(bytes) {
                if let Some(index) = name.rfind('/') {
                    name = &name[index + 1..];
                }
                result.push(NameWithRef {
                    r#ref: oid.to_string(),
                    name: name.to_owned(),
                });
            }
            true
        })?;
        Ok(result)
    }

    pub fn list_tags(&self) -> Result<Vec<String>, git2::Error> {
        Ok(self
            .list_tags_with_refs()?
            .into_iter()
            .map(|n| n.name)
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct NameWithRef {
    pub r#ref: String,
    pub name: String,
}
