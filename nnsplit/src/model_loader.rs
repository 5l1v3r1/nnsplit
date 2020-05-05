use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;

lazy_static! {
    static ref MODEL_DATA: HashMap<&'static str, &'static str> = {
        // this is checked at compile time so a relative path is ok
        let raw_csv = include_str!("../../models.csv");
        let mut model_data = HashMap::new();

        for line in raw_csv.lines() {
            let mut parts = line.split(",");

            model_data.insert(parts.next().unwrap(), parts.next().unwrap());
        }

        model_data
    };
}

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("network error fetching \"{file_name}\" for \"{model_name}\": {source}")]
    NetworkError {
        model_name: String,
        file_name: String,
        source: reqwest::Error,
    },
    #[error("model not found: \"{model_name}\"")]
    ModelNotFoundError { model_name: String },
    #[error(transparent)]
    UrlParseError { source: url::ParseError },
    #[error(transparent)]
    IoError { source: std::io::Error },
}

impl From<url::ParseError> for ResourceError {
    fn from(source: url::ParseError) -> Self {
        ResourceError::UrlParseError { source }
    }
}

impl From<std::io::Error> for ResourceError {
    fn from(source: std::io::Error) -> Self {
        ResourceError::IoError { source }
    }
}

pub fn get_from_cache_or_download(
    model_name: &str,
    file: &str,
) -> Result<impl std::io::Read, ResourceError> {
    let base_url = url::Url::parse(MODEL_DATA.get(model_name).ok_or_else(|| {
        ResourceError::ModelNotFoundError {
            model_name: model_name.to_owned(),
        }
    })?)?;
    let url = base_url.join(file)?;
    let mut cache_path: Option<PathBuf> = None;

    // try to find a file at which to cache the data
    if let Some(project_dirs) = directories::ProjectDirs::from("", "", "nnsplit") {
        let cache_dir = project_dirs.cache_dir();

        cache_path = Some(cache_dir.join(file));
    }

    // if the file can be read, the data is already cached ...
    if let Some(path) = &cache_path {
        if let Ok(bytes) = fs::read(path) {
            return Ok(Cursor::new(bytes));
        }
    }

    // ... otherwise, request the data from the URL ...
    let mut bytes = Vec::new();
    let mut response =
        reqwest::blocking::get(url).map_err(|source| ResourceError::NetworkError {
            model_name: model_name.to_owned(),
            file_name: file.to_owned(),
            source,
        })?;
    response.read_to_end(&mut bytes)?;

    // ... and then cache the data at the provided file, if one was found
    if let Some(path) = &cache_path {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, &bytes)?;
    }

    Ok(Cursor::new(bytes))
}
