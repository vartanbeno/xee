use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub(crate) fn paths(path: &Path) -> Result<(PathBuf, PathBuf)> {
    // look for a directory which contains a `catalog.xml`. This
    // is the first path buf. any remaining path components are
    // a relative path
    for ancestor in path.ancestors() {
        let catalog = ancestor.join("catalog.xml");
        if catalog.exists() {
            let relative = path.strip_prefix(ancestor).unwrap();
            return Ok((catalog, relative.to_path_buf()));
        }
    }
    Err(Error::NoCatalogFound)
}
