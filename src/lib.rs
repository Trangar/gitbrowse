#![warn(missing_docs)]

//! A crate to browse a git repository in a way that most people are used to.
//! Heavily inspired by how people browse a git repository on github and gitlab.
//!
//! ```rust
//! # use gitbrowse::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! let repo = Repo::open(".")?;
//!
//! let branches = repo.list_branches()?;
//! println!("Found the following branches:");
//! for branch in &branches {
//!     println!(" - {}", branch);
//! }
//!
//! # if branches.is_empty() {
//! #     return Ok(());
//! # }
//!
//! let branch = repo.browse_branch(branches.first().unwrap())?;
//! for file in branch.list_files() {
//!     println!("Found file: {:?}", file.path());
//!     println!("File's content is length {}", file.read_content_string()?.len());
//!     
//!     println!("File is modified in the following commits:");
//!     for commit in file.history()? {
//!         if let Ok(commit) = commit {
//!             println!("  {}: {}", commit.id(), commit.message());
//!         }
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```

mod branch;
mod commit;
mod file;
mod repo;
mod utils;

pub use self::branch::Branch;
pub use self::commit::Commit;
pub use self::file::File;
pub use self::repo::Repo;
pub use self::utils::*;
