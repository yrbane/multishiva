use anyhow::{bail, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "multishiva")]
#[command(author = "yrbane")]
#[command(version = "1.0.0")]
#[command(about = "Control multiple computers with one keyboard and mouse", long_about = None)]
pub struct Args {
    /// Mode of operation
    #[arg(short, long, value_enum, env = "MULTISHIVA_MODE")]
    pub mode: Option<Mode>,

    /// Path to configuration file
    #[arg(short, long, env = "MULTISHIVA_CONFIG")]
    pub config: Option<String>,

    /// Launch GUI
    #[arg(long, env = "MULTISHIVA_GUI")]
    pub gui: bool,

    /// Enable simulation mode (for testing)
    #[arg(long, env = "MULTISHIVA_SIMULATE")]
    pub simulate: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum Mode {
    /// Host mode (master)
    Host,
    /// Agent mode (client)
    Agent,
}

impl Args {
    /// Validate argument combinations
    pub fn validate(&self) -> Result<()> {
        // GUI mode and simulate mode are mutually exclusive
        if self.gui && self.simulate {
            bail!("Cannot use --gui and --simulate together");
        }

        // If mode is explicitly set via CLI or env var, ensure it's valid
        if let Some(mode) = &self.mode {
            if self.gui {
                bail!(
                    "Cannot specify --mode {:?} with --gui (GUI will auto-detect mode)",
                    mode
                );
            }
        }

        Ok(())
    }
}

pub fn parse_args() -> Args {
    Args::parse()
}

pub fn parse_and_validate() -> Result<Args> {
    let args = Args::parse();
    args.validate()?;
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_validation_gui_and_simulate_conflict() {
        let args = Args {
            mode: None,
            config: None,
            gui: true,
            simulate: true,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_args_validation_mode_and_gui_conflict() {
        let args = Args {
            mode: Some(Mode::Host),
            config: None,
            gui: true,
            simulate: false,
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_args_validation_valid_host_mode() {
        let args = Args {
            mode: Some(Mode::Host),
            config: Some("config.yml".to_string()),
            gui: false,
            simulate: false,
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validation_valid_agent_mode() {
        let args = Args {
            mode: Some(Mode::Agent),
            config: Some("config.yml".to_string()),
            gui: false,
            simulate: false,
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validation_valid_simulate_mode() {
        let args = Args {
            mode: None,
            config: None,
            gui: false,
            simulate: true,
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validation_valid_gui_mode() {
        let args = Args {
            mode: None,
            config: None,
            gui: true,
            simulate: false,
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_mode_equality() {
        assert_eq!(Mode::Host, Mode::Host);
        assert_eq!(Mode::Agent, Mode::Agent);
        assert_ne!(Mode::Host, Mode::Agent);
    }
}
