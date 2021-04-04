# GitBrowse

[![Rust](https://github.com/Trangar/gitbrowse/actions/workflows/rust.yml/badge.svg)](https://github.com/Trangar/gitbrowse/actions/workflows/rust.yml)

A crate to browse a git repository in a way that most people are used to.
Heavily inspired by how people browse a git repository on github and gitlab.

```rust
let repo = Repo::open(".")?;

let branches = repo.list_branches()?;
println!("Found the following branches:");
for branch in &branches {
    println!(" - {}", branch);
}

let branch = repo.browse_branch(branches.first().unwrap())?;
for file in branch.list_files() {
    println!("Found file: {:?}", file.path());
    println!("File's content is length {}", file.read_content_string()?.len());
    
    println!("File is modified in the following commits:");
    for commit in file.history()? {
        let commit = commit?;
        println!("  {}: {}", commit.id(), commit.message());
    }
}
```