use clap::Parser;

#[derive(Parser)]
#[command(name = "text-to-speech-rs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    Run {
        #[arg(long, default_value_t = true)]
        auto_migrate: bool,
    },
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
}

#[derive(Parser)]
pub enum MigrateCommand {
    Up,
    Status
}