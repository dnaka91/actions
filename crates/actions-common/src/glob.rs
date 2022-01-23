//! Create glob sets to match files against.

use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

/// Create a [`GlobSet`] from the given list of glob patterns.
pub fn build_globset(globs: &[impl AsRef<str>]) -> Result<GlobSet> {
    globs
        .iter()
        .map(|glob| {
            GlobBuilder::new(glob.as_ref())
                .literal_separator(true)
                .build()
        })
        .try_fold(GlobSetBuilder::new(), |mut set, glob| {
            set.add(glob?);
            anyhow::Ok(set)
        })?
        .build()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_globs() -> Result<()> {
        let set = build_globset(&["*.tar.gz", "*.zip"])?;
        assert!(set.is_match("hello.tar.gz"));
        assert!(set.is_match("hello.zip"));
        assert!(!set.is_match("tmp/hello.zip"));
        Ok(())
    }

    #[test]
    fn invalid_glob() {
        assert!(build_globset(&["{]"]).is_err());
    }
}
