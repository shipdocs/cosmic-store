use crate::config::Config;
use crate::gstreamer::Mode;
use clap::Parser;
use cosmic::app::CosmicFlags;
use cosmic::cosmic_config;

#[derive(Debug, Default, Parser)]
pub struct Cli {
    pub subcommand_opt: Option<String>,
    //TODO: should these extra gst-install-plugins-helper arguments actually be handled?
    #[arg(long)]
    pub transient_for: Option<String>,
    #[arg(long)]
    pub interaction: Option<String>,
    #[arg(long)]
    pub desktop_id: Option<String>,
    #[arg(long)]
    pub startup_notification_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub subcommand_opt: Option<String>,
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
    pub mode: Mode,
}

//TODO
impl CosmicFlags for Flags {
    type SubCommand = String;
    type Args = Vec<String>;

    fn action(&self) -> Option<&String> {
        self.subcommand_opt.as_ref()
    }
}
