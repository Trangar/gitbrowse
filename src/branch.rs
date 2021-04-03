use crate::File;

#[allow(dead_code)]
pub struct Branch<'a> {
    pub(crate) repo: &'a git2::Repository,
    pub(crate) branch: git2::Branch<'a>,
    pub(crate) tree: git2::Tree<'a>,
}

impl<'a> Branch<'a> {
    pub(crate) fn new(repo: &'a git2::Repository, branch: &'a str) -> Result<Self, git2::Error> {
        let branch = repo.find_branch(branch, git2::BranchType::Local)?;
        let tree = branch.get().peel_to_tree()?;
        Ok(Branch { repo, branch, tree })
    }

    pub fn list_files(&self, mut cb: impl FnMut(&File)) -> Result<(), git2::Error> {
        self.tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            let file = File {
                branch: self,
                entry: entry.clone(),
            };
            cb(&file);
            git2::TreeWalkResult::Ok
        })?;
        Ok(())
    }

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
