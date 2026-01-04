#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScrollContext {
    NavPage,
    ExplorePage,
    SearchResults,
    DetailsPage,
}

impl ScrollContext {
    pub fn unused_contexts(&self) -> &'static [ScrollContext] {
        // Contexts that can be safely removed when another is active
        match self {
            Self::NavPage => &[Self::DetailsPage, Self::SearchResults, Self::ExplorePage],
            Self::ExplorePage => &[Self::DetailsPage, Self::SearchResults],
            Self::SearchResults => &[Self::DetailsPage],
            Self::DetailsPage => &[],
        }
    }
}
