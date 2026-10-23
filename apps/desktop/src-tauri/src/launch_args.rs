//! GUI launch options of upstream `main.cpp`: `--repo <url-or-file>` loads
//! a custom OS list for this session instead of the official one.

use reqwest::Url;

/// The repository URL from `--repo <value>` or `--repo=<value>`: http(s)
/// URLs as given, anything else a local file turned into a `file://` URL.
///
/// # Errors
/// Upstream's message when the value is not an existing regular file, or
/// when `--repo` has no value.
pub fn repo_from_args(args: &[String]) -> Result<Option<String>, String> {
    let mut value = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--repo" {
            value = Some(iter.next().ok_or("Missing value for --repo")?.clone());
        } else if let Some(v) = arg.strip_prefix("--repo=") {
            value = Some(v.to_owned());
        }
    }
    let Some(repo) = value.filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    if repo.starts_with("http://") || repo.starts_with("https://") {
        return Ok(Some(repo));
    }
    let not_a_file =
        || format!("Custom repository file does not exist or is not a regular file: {repo}");
    let path = std::fs::canonicalize(&repo).map_err(|_| not_a_file())?;
    if !path.is_file() {
        return Err(not_a_file());
    }
    Url::from_file_path(&path)
        .map(|url| Some(url.to_string()))
        .map_err(|()| not_a_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn no_repo_without_the_flag() {
        assert_eq!(repo_from_args(&args(&["--debug"])), Ok(None));
    }

    #[test]
    fn keeps_web_urls() {
        let url = "https://example.com/list.json";
        assert_eq!(
            repo_from_args(&args(&["--repo", url])),
            Ok(Some(url.into()))
        );
        assert_eq!(
            repo_from_args(&args(&[&format!("--repo={url}")])),
            Ok(Some(url.into()))
        );
    }

    #[test]
    fn turns_a_local_file_into_a_file_url() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("my list.json");
        std::fs::write(&file, "{}").unwrap();
        let url = repo_from_args(&args(&["--repo", file.to_str().unwrap()]))
            .unwrap()
            .unwrap();
        assert!(url.starts_with("file:///"), "{url}");
        assert!(url.ends_with("/my%20list.json"), "{url}");
    }

    #[test]
    fn rejects_missing_files_and_directories() {
        let dir = tempfile::tempdir().unwrap();
        for bad in [dir.path().join("nope.json"), dir.path().to_path_buf()] {
            let err = repo_from_args(&args(&["--repo", bad.to_str().unwrap()])).unwrap_err();
            assert!(
                err.starts_with("Custom repository file does not exist"),
                "{err}"
            );
        }
        assert!(repo_from_args(&args(&["--repo"])).is_err());
    }
}
