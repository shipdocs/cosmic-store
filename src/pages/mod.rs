//! Page-related enums for navigation and dialogs

pub mod details;
pub use details::{DetailsPage, DetailsPageActions, SelectedSource};

use crate::Category;
use crate::app_id::AppId;
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

impl NavPage {
    pub fn all() -> &'static [Self] {
        &[
            Self::Explore,
            Self::Create,
            Self::Work,
            Self::Develop,
            Self::Learn,
            Self::Game,
            Self::Relax,
            Self::Socialize,
            Self::Utilities,
            Self::Applets,
            Self::Installed,
            Self::Updates,
        ]
    }

    pub fn title(&self) -> String {
        use crate::fl;
        match self {
            Self::Explore => fl!("explore"),
            Self::Create => fl!("create"),
            Self::Work => fl!("work"),
            Self::Develop => fl!("develop"),
            Self::Learn => fl!("learn"),
            Self::Game => fl!("game"),
            Self::Relax => fl!("relax"),
            Self::Socialize => fl!("socialize"),
            Self::Utilities => fl!("utilities"),
            Self::Applets => fl!("applets"),
            Self::Installed => fl!("installed-apps"),
            Self::Updates => fl!("updates"),
        }
    }

    pub fn categories(&self) -> Option<&'static [Category]> {
        match self {
            Self::Create => Some(&[Category::AudioVideo, Category::Graphics]),
            Self::Work => Some(&[Category::Development, Category::Office, Category::Science]),
            Self::Develop => Some(&[Category::Development]),
            Self::Learn => Some(&[Category::Education]),
            Self::Game => Some(&[Category::Game]),
            Self::Relax => Some(&[Category::AudioVideo]),
            Self::Socialize => Some(&[Category::Network]),
            Self::Utilities => Some(&[Category::Settings, Category::System, Category::Utility]),
            Self::Applets => Some(&[Category::CosmicApplet]),
            _ => None,
        }
    }

    pub fn icon(&self) -> cosmic::widget::icon::Icon {
        use crate::icon_cache::icon_cache_icon;
        match self {
            Self::Explore => icon_cache_icon("store-home-symbolic", 16),
            Self::Create => icon_cache_icon("store-create-symbolic", 16),
            Self::Work => icon_cache_icon("store-work-symbolic", 16),
            Self::Develop => icon_cache_icon("store-develop-symbolic", 16),
            Self::Learn => icon_cache_icon("store-learn-symbolic", 16),
            Self::Game => icon_cache_icon("store-game-symbolic", 16),
            Self::Relax => icon_cache_icon("store-relax-symbolic", 16),
            Self::Socialize => icon_cache_icon("store-socialize-symbolic", 16),
            Self::Utilities => icon_cache_icon("store-utilities-symbolic", 16),
            Self::Applets => icon_cache_icon("store-applets-symbolic", 16),
            Self::Installed => icon_cache_icon("store-installed-symbolic", 16),
            Self::Updates => icon_cache_icon("store-updates-symbolic", 16),
        }
    }
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

impl ExplorePage {
    pub fn all() -> &'static [Self] {
        &[
            Self::EditorsChoice,
            Self::PopularApps,
            Self::MadeForCosmic,
            //TODO: Self::NewApps,
            Self::RecentlyUpdated,
            Self::DevelopmentTools,
            Self::ScientificTools,
            Self::ProductivityApps,
            Self::GraphicsAndPhotographyTools,
            Self::SocialNetworkingApps,
            Self::Games,
            Self::MusicAndVideoApps,
            Self::AppsForLearning,
            Self::Utilities,
        ]
    }

    pub fn title(&self) -> String {
        use crate::fl;
        match self {
            Self::EditorsChoice => fl!("editors-choice"),
            Self::PopularApps => fl!("popular-apps"),
            Self::MadeForCosmic => fl!("made-for-cosmic"),
            Self::NewApps => fl!("new-apps"),
            Self::RecentlyUpdated => fl!("recently-updated"),
            Self::DevelopmentTools => fl!("development-tools"),
            Self::ScientificTools => fl!("scientific-tools"),
            Self::ProductivityApps => fl!("productivity-apps"),
            Self::GraphicsAndPhotographyTools => fl!("graphics-and-photography-tools"),
            Self::SocialNetworkingApps => fl!("social-networking-apps"),
            Self::Games => fl!("games"),
            Self::MusicAndVideoApps => fl!("music-and-video-apps"),
            Self::AppsForLearning => fl!("apps-for-learning"),
            Self::Utilities => fl!("utilities"),
        }
    }

    pub fn categories(&self) -> &'static [Category] {
        match self {
            Self::DevelopmentTools => &[Category::Development],
            Self::ScientificTools => &[Category::Science],
            Self::ProductivityApps => &[Category::Office],
            Self::GraphicsAndPhotographyTools => &[Category::Graphics],
            Self::SocialNetworkingApps => &[Category::Network],
            Self::Games => &[Category::Game],
            Self::MusicAndVideoApps => &[Category::AudioVideo],
            Self::AppsForLearning => &[Category::Education],
            Self::Utilities => &[Category::Settings, Category::System, Category::Utility],
            _ => &[],
        }
    }
}
