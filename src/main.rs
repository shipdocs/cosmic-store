// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use clap::Parser;

mod constants;

mod utils;

mod search;

mod pages;

mod ui;

use cosmic::{
    Application,
    app::Settings,
    cosmic_config::{self, CosmicConfigEntry},
    iced::Limits,
};
use localize::LANGUAGE_SORTER;
use std::collections::BTreeSet;

use app_id::AppId;
mod app_id;

use app_info::{AppIcon, AppInfo, AppUrl};
mod app_info;

use appstream_cache::AppstreamCache;
mod appstream_cache;

mod app_entry;
use app_entry::Apps;

mod backend;

mod cli;
use cli::{Cli, Flags};

use config::{CONFIG_VERSION, Config};
mod config;

mod category;
use category::Category;

mod editors_choice;

use gstreamer::{GStreamerCodec, Mode};
mod gstreamer;

mod icon_cache;

mod key_bind;

mod localize;

#[cfg(feature = "logind")]
mod logind;

mod os_info;

use operation::{Operation, OperationKind, RepositoryRemoveError};
mod operation;

mod priority;

mod scroll_context;
mod search_logic;
mod source;
mod stats;
mod url_handlers;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localize::localize();
    stats::load_stats_async();

    let cli = Cli::parse();

    let (config_handler, config) =
        match cosmic_config::Config::new(app::App::APP_ID, CONFIG_VERSION) {
            Ok(config_handler) => {
                let config = match Config::get_entry(&config_handler) {
                    Ok(ok) => ok,
                    Err((errs, config)) => {
                        log::info!("errors loading config: {:?}", errs);
                        config
                    }
                };
                (Some(config_handler), config)
            }
            Err(err) => {
                log::error!("failed to create config handler: {}", err);
                (None, Config::default())
            }
        };

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(420.0).min_height(300.0));
    settings = settings.exit_on_close(false);

    let mut flags = Flags {
        subcommand_opt: cli.subcommand_opt,
        config_handler,
        config,
        mode: Mode::Normal,
    };

    if let Some(codec) = flags
        .subcommand_opt
        .as_ref()
        .and_then(|x| GStreamerCodec::parse(x))
    {
        // GStreamer installer dialog
        settings = settings.no_main_window(true);
        flags.mode = Mode::GStreamer {
            codec,
            selected: BTreeSet::new(),
            installing: false,
        };
        cosmic::app::run::<app::App>(settings, flags)?;
    } else {
        #[cfg(feature = "single-instance")]
        cosmic::app::run_single_instance::<app::App>(settings, flags)?;

        #[cfg(not(feature = "single-instance"))]
        cosmic::app::run::<app::App>(settings, flags)?;
    }

    Ok(())
}

mod message;
pub use message::{Action, Message};

mod app;
