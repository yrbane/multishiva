use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "multishiva")]
#[command(author = "yrbane")]
#[command(version = "0.1.0")]
#[command(about = "Control multiple computers with one keyboard and mouse", long_about = None)]
pub struct Args {
    /// Mode of operation
    #[arg(short, long, value_enum)]
    pub mode: Option<Mode>,

    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Launch GUI
    #[arg(long)]
    pub gui: bool,

    /// Enable simulation mode (for testing)
    #[arg(long)]
    pub simulate: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Mode {
    /// Host mode (master)
    Host,
    /// Agent mode (client)
    Agent,
}

pub fn parse_args() -> Args {
    Args::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        // Basic smoke test
        assert!(true);
    }
}
