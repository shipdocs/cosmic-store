//! Page-related enums for navigation and dialogs

use crate::app_id::AppId;
use crate::appstream_cache::Category;
use crate::operation::RepositoryRemoveError;
use std::sync::Arc;

use crate::app_info::AppInfo;

/// Context page for the context drawer
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextPage {
    Operations,
    ReleaseNotes(usize, String),
    Repositories,
    Settings,
}

/// Dialog page types
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    FailedOperation(u64),
    RepositoryAddError(String),
    RepositoryRemove(&'static str, RepositoryRemoveError),
    Uninstall(&'static str, AppId, Arc<AppInfo>),
    Place(AppId),
}

/// Navigation page
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub enum NavPage {
    #[default]
    Explore,
    Create,
    Work,
    Develop,
    Learn,
    Game,
    Relax,
    Socialize,
    Utilities,
    Applets,
    Installed,
    Updates,
}

/// Explore page categories
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ExplorePage {
    EditorsChoice,
    PopularApps,
    MadeForCosmic,
    NewApps,
    RecentlyUpdated,
    DevelopmentTools,
    ScientificTools,
    ProductivityApps,
    GraphicsAndPhotographyTools,
    SocialNetworkingApps,
    Games,
    MusicAndVideoApps,
    AppsForLearning,
    Utilities,
}
