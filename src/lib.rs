#![warn(missing_docs)]

//! # GitBrowse
//!
//! A crate to browse a git repository in a way that most people are used to.
//! Heavily inspired by how people browse a git repository on github and gitlab.
//!
//! ```rust,no_run
//! # use gitbrowse::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! let repo = Repo::open("/path/to/some/repo")?;
//! let branch = repo.browse_branch("main")?;
//! branch.list_files(|file| {
//!     println!("Found file: {:?}", file.path());
//!     println!("File's content is length {}", file.read_content_string()?.len());
//!     
//!     println!("File is modified in the following commits:");
//!     file.history(|commit| {
//!         println!("  {}: {}", commit.id(), commit.message());
//!     })?;
//!
//!     Ok(())
//! })?;
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
