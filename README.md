[![Crates.io](https://img.shields.io/crates/v/gitbrowse.svg)](https://crates.io/crates/gitbrowse)
[![docs.rs](https://docs.rs/gitbrowse/badge.svg)](https://docs.rs/gitbrowse)
[![Workflow Status](https://github.com/Trangar/gitbrowse/workflows/Rust/badge.svg)](https://github.com/Trangar/gitbrowse/actions?query=workflow%3A%22Rust%22)

# gitbrowse

A crate to browse a git repository in a way that most people are used to.
Heavily inspired by how people browse a git repository on github and gitlab.

```rust

let repo = Repo::open(".")?;

let branches = repo.list_branches()?;
println!("Found the following branches:");
for branch in &branches {
    println!(" - {}", branch);
}

let current_branch = match repo.current_branch()? {
    Some(b) => b,
    None => return Ok(())
};
println!("Current branch: {:?}", current_branch.name());

for file in current_branch.files() {
    println!("Found file: {:?}", file.path());
    println!("File's content is length {}", file.read_content_string()?.len());

    println!("File is modified in the following commits:");
    for commit in file.history()? {
        if let Ok(commit) = commit {
            println!("  {}: {}", commit.id(), commit.message());
        }
    }
}

```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
