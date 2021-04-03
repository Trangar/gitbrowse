use crate::File;

pub struct Commit<'a> {
    // browse: &'a GitBrowse<'a>,
    commit: git2::Commit<'a>,
}

impl<'a> Commit<'a> {
    pub(crate) fn from_file(_file: &'a File<'a>, commit: git2::Commit<'a>) -> Self {
        Self {
            // browse: file.browse,
            commit,
        }
    }

    pub fn id(&self) -> String {
        let short_id = self.commit.as_object().short_id().unwrap();
        short_id.as_str().unwrap().to_owned()
    }

    pub fn message(&self) -> &str {
        self.commit.message_raw().unwrap()
    }
}
