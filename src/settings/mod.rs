use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid search template: missing query placeholder")]
    InvalidSearchTemplate,
}

pub fn search_template_from_env() -> Result<Option<String>, ConfigError> {
    let template = match std::env::var("SEEKX_SEARCH_URL_TEMPLATE") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => return Ok(None),
    };

    if template.contains("{query}") || template.contains("%s") {
        return Ok(Some(template));
    }

    // allow appending query when no placeholder is present
    Ok(Some(template))
}
