use std::{borrow::Cow, fs::File, io::Read, path::PathBuf};
use zip::ZipArchive;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, Clone)]
pub struct ZipFileContents<'a> {
    /// filename
    pub filename: Cow<'a, str>,
    /// file buffer
    pub buffer: Vec<u8>,
    /// Zip password if any/identified
    pub zip_password: Option<Cow<'a, str>>,
}

/// Extract all files from a given Zip file
/// Zipfile can be password protected or not. This fn can handle both types
///
/// Parameters: path/to/zip-file, Password List
/// Returns: Error or ZipFileContents Struct
///
/// Example
/// ```no_run
/// use xtract::{from_zipfile, Result};
///
/// #[tokio::main]
/// async fn main() {
///     let fname = "/path/to/zip";
///     let passwords = vec!["test".to_string(), "test123".to_string(), "etc".to_string()];
///
///     // With a password list
///     let zip = from_zipfile(fname.to_string(), Some(passwords)).await?;
///
///     // If no password
///     let zip = from_zipfile(fname.to_string(), None).await?;
/// }
/// ```
pub async fn from_zipfile<P: Into<PathBuf> + Send + Sync + Clone>(
    zip_file: P,
    password_list: Option<Vec<String>>,
) -> Result<Vec<ZipFileContents<'static>>> {
    let mut threads: Vec<
        tokio::task::JoinHandle<std::result::Result<Vec<ZipFileContents<'_>>, Error>>,
    > = vec![];
    let password_list = if let Some(mut passwords) = password_list {
        // add for empty/none password
        passwords.push("".to_string());
        // sort and dedupe the vec
        passwords.sort();
        passwords.dedup();
        passwords
    } else {
        vec!["".to_string()]
    };

    for pass in password_list {
        let zipp_file: PathBuf = zip_file.clone().into();
        threads.push(tokio::task::spawn(async move {
            let zipfile = File::open(zipp_file)?;
            let mut zip = ZipArchive::new(zipfile)?;
            (0..zip.len()).try_fold(vec![], move |mut zfc_vec, i| {
                let mut file = zip.by_index_decrypt(i, pass.as_bytes())?;
                let mut f_buf = vec![];
                file.read_to_end(&mut f_buf)?;
                zfc_vec.push(ZipFileContents {
                    filename: Cow::from(file.name().to_string()),
                    buffer: f_buf,
                    zip_password: if pass.is_empty() {
                        None
                    } else {
                        Some(Cow::from(pass.to_string()))
                    },
                });
                Ok::<_, Error>(zfc_vec)
            })
        }));
    }
    let mut err = Error::CannotDecrypt;
    for thread_handle in threads {
        match thread_handle.await {
            Ok(Ok(res)) => return Ok(res),
            Err(e) => {
                err = e.into();
            }
            Ok(Err(e)) => err = e,
        }
    }
    Err(err)
}

/// Get total files in the zip file
pub fn get_total_files<P: Into<PathBuf>>(zipfile: P) -> Result<usize> {
    let file = File::open(zipfile.into())?;
    let archive = ZipArchive::new(file)?;

    Ok(archive.len())
}
