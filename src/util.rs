use crate::Result;

#[derive(thiserror::Error, Debug)]
pub enum MavenError {
    #[error("Unable to find package for library {0}")]
    Package(String),
    #[error("Unable to find name for library {0}")]
    Name(String),
    #[error("Unable to find version for library {0}")]
    Version(String),
    #[error("Unable to find data for library {0}")]
    Data(String),
}

/// Converts a maven artifact identifier to a path.
pub(crate) fn get_path_from_artifact(artifact: &str) -> core::result::Result<String, MavenError> {
    let artifact_ext: Vec<&str> = artifact.rsplitn(2, '@').collect();
    let artifact = artifact_ext.last().unwrap();
    let ext = if artifact_ext.len() == 2 {
        artifact_ext.first().unwrap()
    } else {
        "jar"
    };

    let name_items = artifact.splitn(4, ':').collect::<Vec<&str>>();

    let package = name_items
        .get(0)
        .ok_or_else(|| MavenError::Package(artifact.to_string()))?;
    let name = name_items
        .get(1)
        .ok_or_else(|| MavenError::Name(artifact.to_string()))?;
    let version = name_items
        .get(2)
        .ok_or_else(|| MavenError::Version(artifact.to_string()))?;

    let package = package.replace(".", "/");

    if name_items.len() == 3 {
        Ok(format!(
            "{}/{}/{}/{}-{}.{}",
            package, name, version, name, version, ext
        ))
    } else {
        let data = name_items
            .get(3)
            .ok_or_else(|| MavenError::Data(artifact.to_string()))?;

        Ok(format!(
            "{}/{}/{}/{}-{}-{}.{}",
            package, name, version, name, version, data, ext
        ))
    }
}

/// Computes a checksum of the input bytes
pub(crate) async fn sha1_hash(bytes: bytes::Bytes) -> Result<String> {
    let hash = tokio::task::spawn_blocking(|| sha1::Sha1::from(bytes).hexdigest()).await?;

    Ok(hash)
}
