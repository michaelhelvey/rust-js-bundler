/// Extremely minimalistic resolver that implements some tiny fraction of the
/// real Node.js module resolution algorithm for ESM.
use color_eyre::{eyre::eyre, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug)]
pub struct ImportStatement {
    /// The module specifier in the import.  e.g. the `path` in `require('path')
    /// or `import path from 'path'`
    pub specifier: String,

    /// The path to the file that the import is relative to (the file that
    /// contains the import)
    pub relative_to: String,
}

/// Given a file, return a list of all the import statements
pub async fn get_import_statements(mut file: File, path: &String) -> Result<Vec<ImportStatement>> {
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).await?;

    lazy_static! {
        static ref IMPORT_RE: Regex =
            Regex::new("import .* from ['\"](\\.?[\\w\\.\\/]+)['\"]").unwrap();
    }

    let captures = IMPORT_RE
        .captures_iter(&file_contents)
        .map(|cap| ImportStatement {
            specifier: cap.get(1).unwrap().as_str().to_string(),
            relative_to: path.clone(),
        })
        .collect::<Vec<_>>();

    Ok(captures)
}

/// Implements the Node.js module resolution algorithm for ESM Given a module
/// specifier and the path to the file that contains the import, e.g. import xyz
/// from './foo.js' relative to module '/home/user/bar.js', it will resolve
/// './foo.js' to '/home/user/foo.js'.
///
/// This is a very minimalistic implementation that only implements a tiny
/// fraction of the real algorithm.
///
/// See: https://nodejs.org/api/modules.html#modules_all_together
pub async fn resolve_import(import: &String, from: &String) -> Result<String> {
    let directory = std::path::Path::new(from).parent().unwrap();
    let file_path = PathBuf::from(import);

    Ok(directory
        .join(file_path)
        .canonicalize()
        .map_err(|e| {
            eyre!(
                "Failed to resolve import of '{}' from '{}': {}",
                import,
                from,
                e
            )
        })?
        .to_string_lossy()
        .to_string())
}
