# GitBrowse

A crate to browse a git repository in a way that most people are used to.
Heavily inspired by how people browse a git repository on github and gitlab.

```rust
use gitbrowse::*;

let repo = Repo::open(".")?;
let branch = repo.browse_branch("main")?;
for file in branch.list_files() {
    println!("Found file: {:?}", file.path());
    println!("File content as a string has length {}", file.read_content_string()?.len());
    
    println!("File is modified in the following commits:");
    for commit in file.history()? {
        let commit = commit?;
        println!("  {}: {}", commit.id(), commit.message());
    }
}
```