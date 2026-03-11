#[derive(Clone, Debug)]
pub struct DesktopApp {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub search_terms: Vec<String>,
    pub normalized_terms: Vec<String>,
}
