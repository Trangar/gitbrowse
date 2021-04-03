# GitBrowse

A crate to browse a git repository in a way that most people are used to.
Heavily inspired by how people browse a git repository on github and gitlab.

```rust
let repo = Repo::open("/path/to/some/repo")?;
let branch = repo.browse_branch("main")?;
branch.list_files(|file| {
    println!("Found file: {:?}", file.path());
    println!("File's content is length {}", file.read_content_string()?.len());
    
    println!("File is modified in the following commits:");
    file.history(|commit| {
        println!("  {}: {}", commit.id(), commit.message());
    })?;

    Ok(())
})?;
```
