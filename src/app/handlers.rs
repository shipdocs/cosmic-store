use crate::app::{App, Mode};
use crate::message::Message;
use crate::operation::{Operation, OperationKind, RepositoryRemoveError};
use crate::pages::{DialogPage, NavPage};
use cosmic::app::Task;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::futures::SinkExt;
use cosmic::iced::keyboard::{self, Key};
use cosmic::iced::window;
use cosmic::iced::{Subscription, futures, stream};
use cosmic::widget;
use cosmic::{Application, action};
use std::env;
use std::future::pending;
use std::process;

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::AppTheme(_) | Message::Config(_) | Message::SystemThemeModeChange(_) => {
            return app.handle_config_message(message);
        }
        Message::LoadingTick => {
            if matches!(app.mode, Mode::Normal) {
                app.loading_frame = app.loading_frame.wrapping_add(1);
            }
            return Task::none();
        }
        Message::Apps(apps) => {
            app.apps = apps;
            return Task::none();
        }
        Message::Backends(_)
        | Message::StatsLoaded(_)
        | Message::CheckUpdates
        | Message::UpdateAll
        | Message::Updates(_) => {
            return app.handle_backend_message(message);
        }
        Message::DialogCancel | Message::DialogConfirm | Message::DialogPage(_) => {
            return app.handle_dialog_message(message);
        }
        Message::Operation(_, _, _, _)
        | Message::PendingComplete(_)
        | Message::PendingDismiss
        | Message::PendingError(_, _)
        | Message::PendingProgress(_, _)
        | Message::RepositoryAdd(_, _)
        | Message::RepositoryAddDialog(_) => {
            return app.handle_operation_message(message);
        }
        Message::CategoryResults(_, _)
        | Message::SearchActivate
        | Message::SearchClear
        | Message::SearchInput(_)
        | Message::SearchResults(..)
        | Message::SearchSortMode(_)
        | Message::SearchSubmit(_)
        | Message::WaylandFilter(_) => {
            return app.handle_search_message(message);
        }
        Message::Select(_, _, _, _)
        | Message::SelectInstalled(_)
        | Message::SelectUpdates(_)
        | Message::SelectNone
        | Message::SelectCategoryResult(_)
        | Message::SelectExploreResult(_, _)
        | Message::SelectSearchResult(_)
        | Message::SelectedAddonsViewMore(_)
        | Message::SelectedScreenshot(..)
        | Message::SelectedScreenshotShown(_)
        | Message::SelectedSource(_) => {
            return app.handle_selection_message(message);
        }
        Message::RepositoryRemove(backend_name, repo_rms) => {
            app.operation(Operation {
                kind: OperationKind::RepositoryRemove(repo_rms, false),
                backend_name,
                package_ids: Vec::new(),
                infos: Vec::new(),
            });
        }
        Message::ToggleUninstallPurgeData(value) => {
            app.uninstall_purge_data = value;
        }
        Message::ExplorePage(explore_page_opt) => {
            app.explore_page_opt = explore_page_opt;
            return app.update_scroll();
        }
        Message::ExploreResults(explore_page, results) => {
            // Load icons lazily when results are received (not during search)
            let mut results = results;
            app.load_icons_for_results(&mut results);
            app.explore_results.insert(explore_page, results);
        }
        Message::ExploreResultsReady(results_map) => {
            // Batch results received - load icons and insert all at once
            for (explore_page, mut results) in results_map {
                app.load_icons_for_results(&mut results);
                app.explore_results.insert(explore_page, results);
            }
        }
        Message::GStreamerExit(code) => match app.mode {
            Mode::Normal => {}
            Mode::GStreamer { .. } => {
                process::exit(code as i32);
            }
        },
        Message::GStreamerInstall => {
            let mut ops = Vec::new();
            match &mut app.mode {
                Mode::Normal => {}
                Mode::GStreamer {
                    selected,
                    installing,
                    ..
                } => {
                    if let Some((_input, results)) = &app.search_results {
                        for (i, result) in results.iter().enumerate() {
                            let installed = App::is_installed_inner(
                                &app.installed,
                                result.backend_name(),
                                &result.id,
                                &result.info,
                            );
                            if installed != selected.contains(&i) {
                                let kind = if installed {
                                    OperationKind::Uninstall { purge_data: false }
                                } else {
                                    OperationKind::Install
                                };
                                eprintln!(
                                    "{:?} {:?} from backend {} and info {:?}",
                                    kind,
                                    result.id,
                                    result.backend_name(),
                                    result.info
                                );
                                ops.push(Operation {
                                    kind,
                                    backend_name: result.backend_name(),
                                    package_ids: vec![result.id.clone()],
                                    infos: vec![result.info.clone()],
                                });
                            }
                        }
                        *installing = true;
                    }
                }
            }
            for op in ops {
                app.operation(op);
            }
        }
        Message::GStreamerToggle(i) => match &mut app.mode {
            Mode::Normal => {}
            Mode::GStreamer { selected, .. } => {
                if !selected.remove(&i) {
                    selected.insert(i);
                }
            }
        },
        Message::Installed(installed) => {
            app.installed = Some(installed);
            app.waiting_installed.clear();

            app.update_apps();
            let mut commands = Vec::new();
            if app.search_active && app.details_page_opt.is_none() {
                commands.push(app.search());
            }
            match app.mode {
                Mode::Normal => {
                    if let Some(categories) = app
                        .nav_model
                        .active_data::<NavPage>()
                        .and_then(|nav_page| nav_page.categories())
                    {
                        commands.push(app.categories(categories));
                    }
                    commands.push(app.installed_results());
                    // Batch all explore page searches into a single O(N) pass instead of O(13N)
                    commands.push(app.explore_results_all_batch());
                }
                Mode::GStreamer { .. } => {}
            }
            return Task::batch(commands);
        }
        Message::InstalledResults(installed_results) => {
            // Load icons lazily when results are received (not during search)
            let mut installed_results = installed_results;
            app.load_icons_for_results(&mut installed_results);
            app.installed_results = Some(installed_results);
        }
        Message::Key(modifiers, key, text) => {
            if !app.dialog_pages.is_empty()
                && matches!(key, Key::Named(keyboard::key::Named::Escape))
                && !modifiers.logo()
                && !modifiers.control()
                && !modifiers.alt()
                && !modifiers.shift()
            {
                return update(app, Message::DialogCancel);
            }

            for (key_bind, action) in app.key_binds.iter() {
                if key_bind.matches(modifiers, &key) {
                    return update(app, action.message());
                }
            }

            if !modifiers.logo()
                && !modifiers.control()
                && !modifiers.alt()
                && matches!(key, Key::Character(_))
            {
                if let Some(text) = text {
                    app.search_active = true;
                    app.search_input.push_str(&text);
                    return Task::batch([
                        widget::text_input::focus(app.search_id.clone()),
                        app.search(),
                    ]);
                }
            }
        }
        Message::LaunchUrl(url) => match open::that_detached(&url) {
            Ok(()) => {}
            Err(err) => {
                log::warn!("failed to open {:?}: {}", url, err);
            }
        },
        Message::MaybeExit => {
            if app.core.main_window_id().is_none() && app.pending_operations.is_empty() {
                process::exit(0);
            }
        }
        #[cfg(feature = "notify")]
        Message::Notification(notification) => {
            app.notification_opt = Some(notification);
        }
        Message::OpenDesktopId(desktop_id) => {
            return app.open_desktop_id(desktop_id);
        }
        Message::ScrollView(viewport) => {
            app.scroll_views.insert(app.scroll_context(), viewport);
        }
        Message::ToggleContextPage(context_page) => {
            if app.core.window.show_context && app.context_page == context_page {
                app.core.window.show_context = false;
            } else {
                app.context_page = context_page;
                app.core.window.show_context = true;
            }
        }
        Message::WindowClose => {
            if let Some(window_id) = app.core.main_window_id() {
                app.core.set_main_window_id(None);
                return Task::batch([
                    window::close(window_id),
                    Task::perform(async move { action::app(Message::MaybeExit) }, |x| x),
                ]);
            }
        }
        Message::WindowNew => match env::current_exe() {
            Ok(exe) => match process::Command::new(&exe).spawn() {
                Ok(_child) => {}
                Err(err) => {
                    log::error!("failed to execute {:?}: {}", exe, err);
                }
            },
            Err(err) => {
                log::error!("failed to get current executable path: {}", err);
            }
        },
        Message::SelectPlacement(selection) => {
            app.applet_placement_buttons.activate(selection);
        }
        #[cfg(not(feature = "wayland"))]
        Message::PlaceApplet(id) => {
            log::error!(
                "cannot place applet {:?}, not compiled with wayland feature",
                id
            );
        }
        #[cfg(feature = "wayland")]
        Message::PlaceApplet(id) => {
            app.dialog_pages.pop_front();

            // Panel or Dock specific references
            let panel_info = if Some(app.applet_placement_buttons.active())
                == app.applet_placement_buttons.entity_at(1)
            {
                ("Dock", "cosmic-settings dock-applet")
            } else {
                ("Panel", "cosmic-settings panel-applet")
            };

            // Load in panel or dock configs for adding new applet
            let panel_config_helper =
                cosmic_panel_config::CosmicPanelConfig::cosmic_config(panel_info.0).ok();
            let mut applet_config = panel_config_helper
                .as_ref()
                .and_then(|panel_config_helper| {
                    let panel_config =
                        cosmic_panel_config::CosmicPanelConfig::get_entry(panel_config_helper)
                            .ok()?;
                    (panel_config.name == panel_info.0).then_some(panel_config)
                });
            let Some(applet_config) = applet_config.as_mut() else {
                return Task::none();
            };

            // check if the applet is already added to the panel
            let applet_id = id.raw().to_owned();
            let mut applet_exists = false;
            if let Some(center) = applet_config.plugins_center.as_ref() {
                if center.iter().any(|a: &String| a.as_str() == applet_id) {
                    applet_exists = true;
                }
            }
            if let Some(wings) = applet_config.plugins_wings.as_ref() {
                if wings
                    .0
                    .iter()
                    .chain(wings.1.iter())
                    .any(|a: &String| a.as_str() == applet_id)
                {
                    applet_exists = true;
                }
            }

            // if applet doesn't already exist, continue adding
            if !applet_exists {
                // add applet to the end of the left wing (matching the applet settings behaviour)
                let list = if let Some((list, _)) = applet_config.plugins_wings.as_mut() {
                    list
                } else {
                    applet_config.plugins_wings = Some((Vec::new(), Vec::new()));
                    &mut applet_config.plugins_wings.as_mut().unwrap().0
                };
                list.push(id.raw().to_string());

                // save config
                if let Some(save_helper) = panel_config_helper.as_ref() {
                    if let Err(e) = applet_config.write_entry(save_helper) {
                        log::error!("Failed to save applet: {:?}", e);
                    }
                } else {
                    log::error!("No panel config helper. Failed to save applet.");
                };
            }

            // launch the applet settings
            let settings_desktop_id = "com.system76.CosmicSettings";
            let exec = panel_info.1;
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || Some((exec, settings_desktop_id)))
                        .await
                        .unwrap_or(None)
                },
                |result| {
                    #[cfg(feature = "desktop")]
                    if let Some((exec, settings_desktop_id)) = result {
                        tokio::spawn(async move {
                            cosmic::desktop::spawn_desktop_exec(
                                &exec,
                                Vec::<(&str, &str)>::new(),
                                Some(settings_desktop_id),
                                false,
                            )
                            .await;
                        });
                    }
                    action::none()
                },
            );
        }
    }

    Task::none()
}

pub fn subscription(app: &App) -> Subscription<Message> {
    let mut subscriptions = vec![
        cosmic::iced::event::listen_with(|event, status, _window_id| match event {
            cosmic::iced::event::Event::Keyboard(cosmic::iced::keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) => match status {
                cosmic::iced::event::Status::Ignored => Some(Message::Key(modifiers, key, text)),
                cosmic::iced::event::Status::Captured => None,
            },
            _ => None,
        }),
        cosmic::cosmic_config::config_subscription(
            std::any::TypeId::of::<crate::config::Config>(),
            crate::app::App::APP_ID.into(),
            crate::config::CONFIG_VERSION,
        )
        .map(|update| {
            if !update.errors.is_empty() {
                log::debug!("errors loading config: {:?}", update.errors);
            }
            Message::Config(update.config)
        }),
        cosmic::cosmic_config::config_subscription::<_, cosmic::cosmic_theme::ThemeMode>(
            std::any::TypeId::of::<cosmic::cosmic_theme::ThemeMode>(),
            cosmic::cosmic_theme::THEME_MODE_ID.into(),
            cosmic::cosmic_theme::ThemeMode::version(),
        )
        .map(|update| {
            if !update.errors.is_empty() {
                log::debug!("errors loading theme mode: {:?}", update.errors);
            }
            Message::SystemThemeModeChange(update.config)
        }),
    ];

    /*
    #[cfg(feature = "logind")]
    if let Some(logind) = &app.core.logind {
        subscriptions.push(crate::logind::logind_subscription(logind));
    }
    */

    if app.explore_results.is_empty() {
        subscriptions.push(
            cosmic::iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| Message::LoadingTick),
        );
    }

    /*
    if let Some(notification) = &app.notification_opt {
        subscriptions.push(
            stream::channel(16, {
                let notification = notification.clone();
                move |mut output| async move {
                    let mut notification = notification.lock().unwrap();
                    while let Some(action) = notification.action().await {
                        let _ = output.send(Message::LaunchUrl(action.into())).await;
                    }
                    pending().await
                }
            })
            .into(),
        );
    }
    */

    if !app.pending_operations.is_empty() {
        #[cfg(feature = "logind")]
        {
            struct InhibitSubscription;
            subscriptions.push(Subscription::run_with_id(
                std::any::TypeId::of::<InhibitSubscription>(),
                stream::channel(1, move |_msg_tx| async move {
                    let _inhibits = crate::logind::inhibit().await;
                    pending().await
                }),
            ));
        }
    }

    for (id, (op, _progress)) in app.pending_operations.iter() {
        if app.progress_operations.contains(id) {
            continue;
        }

        let id = *id;
        let op = op.clone();
        //let msg_tx = app.core.message_sender.clone();
        let backend = app.backends.get(&op.backend_name).cloned();

        subscriptions.push(Subscription::run_with_id(
            id,
            stream::channel(16, move |mut msg_tx_stream| async move {
                let res = match backend {
                    Some(backend) => {
                        let on_progress = {
                            let mut msg_tx = msg_tx_stream.clone();
                            Box::new(move |progress| {
                                let _ = futures::executor::block_on(async {
                                    msg_tx.send(Message::PendingProgress(id, progress)).await
                                });
                            })
                        };
                        let mut msg_tx = msg_tx_stream.clone();
                        tokio::task::spawn_blocking(move || {
                            match backend.operation(&op, on_progress) {
                                Ok(()) => Ok(()),
                                Err(err) => match err.downcast_ref::<RepositoryRemoveError>() {
                                    Some(repo_rm) => {
                                        let _ = futures::executor::block_on(async {
                                            msg_tx
                                                .send(Message::DialogPage(
                                                    DialogPage::RepositoryRemove(
                                                        op.backend_name,
                                                        repo_rm.clone(),
                                                    ),
                                                ))
                                                .await
                                        });
                                        Ok(())
                                    }
                                    None => Err(err.to_string()),
                                },
                            }
                        })
                        .await
                        .unwrap()
                    }
                    None => Err(format!("backend {:?} not found", op.backend_name)),
                };

                match res {
                    Ok(()) => {
                        let _ = msg_tx_stream.send(Message::PendingComplete(id)).await;
                    }
                    Err(err) => {
                        let _ = msg_tx_stream.send(Message::PendingError(id, err)).await;
                    }
                }
                pending().await
            }),
        ));
    }

    if let Some(selected) = &app.details_page_opt {
        for (screenshot_i, screenshot) in selected.info.screenshots.iter().enumerate() {
            let url = screenshot.url.clone();
            subscriptions.push(Subscription::run_with_id(
                url.clone(),
                stream::channel(16, move |mut msg_tx| async move {
                    log::info!("fetch screenshot {}", url);
                    match reqwest::get(&url).await {
                        Ok(response) => match response.bytes().await {
                            Ok(bytes) => {
                                log::info!(
                                    "fetched screenshot from {}: {} bytes",
                                    url,
                                    bytes.len()
                                );
                                let _ = msg_tx
                                    .send(Message::SelectedScreenshot(
                                        screenshot_i,
                                        url,
                                        bytes.to_vec(),
                                    ))
                                    .await;
                            }
                            Err(err) => {
                                log::warn!("failed to read screenshot from {}: {}", url, err);
                            }
                        },
                        Err(err) => {
                            log::warn!("failed to request screenshot from {}: {}", url, err);
                        }
                    }
                    pending().await
                }),
            ));
        }
    }

    Subscription::batch(subscriptions)
}
