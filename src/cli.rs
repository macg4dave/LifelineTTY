use crate::{config::Pcf8574Addr, Error, Result};

/// Entry mode for the `run` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Default daemon path that renders onto the LCD.
    Daemon,
    /// P7: CLI integration groundwork for the serial shell preview gate.
    SerialShell,
}

impl Default for RunMode {
    fn default() -> Self {
        RunMode::Daemon
    }
}

/// Options for the `run` command; values are `None` when not provided on CLI.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunOptions {
    pub mode: RunMode,
    pub device: Option<String>,
    pub baud: Option<u32>,
    pub cols: Option<u8>,
    pub rows: Option<u8>,
    pub payload_file: Option<String>,
    pub backoff_initial_ms: Option<u64>,
    pub backoff_max_ms: Option<u64>,
    pub pcf8574_addr: Option<Pcf8574Addr>,
    pub log_level: Option<String>,
    pub log_file: Option<String>,
        pub fn help() -> String {
            let mut help = String::from(
                "lifelinetty - Serial-to-LCD daemon\n\nUSAGE:\n  lifelinetty run [--device <path>] [--baud <number>] [--cols <number>] [--rows <number>] [--payload-file <path>]\n  lifelinetty --help\n  lifelinetty --version\n\nOPTIONS:\n  --device <path>   Serial device path (default: /dev/ttyUSB0)\n  --baud <number>   Baud rate (default: 9600)\n  --cols <number>   LCD columns (default: 20)\n  --rows <number>   LCD rows (default: 4)\n  --payload-file <path>  Load a local JSON payload and render it once (testing helper)\n  --backoff-initial-ms <number>  Initial reconnect backoff (default: 500)\n  --backoff-max-ms <number>      Maximum reconnect backoff (default: 10000)\n  --pcf8574-addr <auto|0xNN>     PCF8574 I2C address or 'auto' to probe (default: auto)\n  --log-level <error|warn|info|debug|trace>  Log verbosity (default: info)\n  --log-file <path>              Append logs inside /run/serial_lcd_cache (also honors LIFELINETTY_LOG_PATH)\n  --demo                         Run built-in demo pages on the LCD (no serial input)\n",
            );

            #[cfg(feature = "serialsh-preview")]
            {
                help.push_str("  --serialsh                   Preview: enable the serial shell mode (feature-gated, incomplete)\n");
            }

            help.push_str("  -h, --help        Show this help\n  -V, --version     Show version\n");
            help
        }
            "lifelinetty - Serial-to-LCD daemon\n",
            "\n",
            "USAGE:\n",
            "  lifelinetty run [--device <path>] [--baud <number>] [--cols <number>] [--rows <number>] [--payload-file <path>]\n",
            "  lifelinetty --help\n",
            "  lifelinetty --version\n",
            "\n",
            "OPTIONS:\n",
            "  --device <path>   Serial device path (default: /dev/ttyUSB0)\n",
            "  --baud <number>   Baud rate (default: 9600)\n",
            "  --cols <number>   LCD columns (default: 20)\n",
            "  --rows <number>   LCD rows (default: 4)\n",
            "  --payload-file <path>  Load a local JSON payload and render it once (testing helper)\n",
            "  --backoff-initial-ms <number>  Initial reconnect backoff (default: 500)\n",
            "  --backoff-max-ms <number>      Maximum reconnect backoff (default: 10000)\n",
            "  --pcf8574-addr <auto|0xNN>     PCF8574 I2C address or 'auto' to probe (default: auto)\n",
            "  --log-level <error|warn|info|debug|trace>  Log verbosity (default: info)\n",
            "  --log-file <path>              Append logs inside /run/serial_lcd_cache (also honors LIFELINETTY_LOG_PATH)\n",
            "  --demo                         Run built-in demo pages on the LCD (no serial input)\n",
            "  -h, --help        Show this help\n",
            "  -V, --version     Show version\n",
        )
=======
    pub fn help() -> String {
        let mut help = String::from(
            "lifelinetty - Serial-to-LCD daemon\n\nUSAGE:\n  lifelinetty run [--device <path>] [--baud <number>] [--cols <number>] [--rows <number>] [--payload-file <path>]\n  lifelinetty --help\n  lifelinetty --version\n\nOPTIONS:\n  --device <path>   Serial device path (default: /dev/ttyUSB0)\n  --baud <number>   Baud rate (default: 9600)\n  --cols <number>   LCD columns (default: 20)\n  --rows <number>   LCD rows (default: 4)\n  --payload-file <path>  Load a local JSON payload and render it once (testing helper)\n  --backoff-initial-ms <number>  Initial reconnect backoff (default: 500)\n  --backoff-max-ms <number>      Maximum reconnect backoff (default: 10000)\n  --pcf8574-addr <auto|0xNN>     PCF8574 I2C address or 'auto' to probe (default: auto)\n  --log-level <error|warn|info|debug|trace>  Log verbosity (default: info)\n  --log-file <path>              Append logs inside /run/serial_lcd_cache (also honors LIFELINETTY_LOG_PATH)\n  --demo                         Run built-in demo pages on the LCD (no serial input)\n",
        );

        #[cfg(feature = "serialsh-preview")]
        {
            help.push_str(
                "  --serialsh                   Preview: enable the serial shell mode (feature-gated, incomplete)\n",
            );
        }

        help.push_str("  -h, --help        Show this help\n  -V, --version     Show version\n");
        help
>>>>>>> fa94790 (feat: Implement serialsh preview mode with CLI integration and command tunnel scaffolding)
    }

    

    pub fn print_help() {
        println!("{}", Self::help());
    }
}

fn parse_run_options(iter: &mut std::slice::Iter<String>) -> Result<RunOptions> {
    let mut opts = RunOptions::default();

    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--device" => {
                opts.device = Some(take_value(flag, iter)?);
            }
            "--baud" => {
                let raw = take_value(flag, iter)?;
                opts.baud = Some(raw.parse().map_err(|_| {
                    Error::InvalidArgs("baud must be a positive integer".to_string())
                })?);
            }
            "--cols" => {
                let raw = take_value(flag, iter)?;
                opts.cols = Some(raw.parse().map_err(|_| {
                    Error::InvalidArgs("cols must be a positive integer".to_string())
                })?);
            }
            "--rows" => {
                let raw = take_value(flag, iter)?;
                opts.rows = Some(raw.parse().map_err(|_| {
                    Error::InvalidArgs("rows must be a positive integer".to_string())
                })?);
            }
            "--payload-file" => {
                opts.payload_file = Some(take_value(flag, iter)?);
            }
            "--backoff-initial-ms" => {
                let raw = take_value(flag, iter)?;
                opts.backoff_initial_ms = Some(raw.parse().map_err(|_| {
                    Error::InvalidArgs("backoff-initial-ms must be a positive integer".to_string())
                })?);
            }
            "--backoff-max-ms" => {
                let raw = take_value(flag, iter)?;
                opts.backoff_max_ms = Some(raw.parse().map_err(|_| {
                    Error::InvalidArgs("backoff-max-ms must be a positive integer".to_string())
                })?);
            }
            "--pcf8574-addr" => {
                let raw = take_value(flag, iter)?;
                opts.pcf8574_addr = Some(raw.parse().map_err(|_| {
                    Error::InvalidArgs(
                        "pcf8574-addr must be 'auto' or a hex/decimal address (e.g., 0x27)"
                            .to_string(),
                    )
                })?);
            }
            "--log-level" => {
                opts.log_level = Some(take_value(flag, iter)?);
            }
            "--log-file" => {
                opts.log_file = Some(take_value(flag, iter)?);
            }
            "--demo" => {
                opts.demo = true;
            }
            #[cfg(feature = "serialsh-preview")]
            "--serialsh" => {
                // P7: expose the serial shell gate while milestone A wiring lands.
                opts.mode = RunMode::SerialShell;
            }
            }
            other => {
                return Err(Error::InvalidArgs(format!(
                    "unknown flag '{other}', try --help"
                )));
            }
        }
    }

    validate_serialsh_options(&opts)?;
    Ok(opts)
}

fn take_value(flag: &str, iter: &mut std::slice::Iter<String>) -> Result<String> {
    iter.next()
        .cloned()
        .ok_or_else(|| Error::InvalidArgs(format!("expected a value after {flag}")))
}

#[cfg(feature = "serialsh-preview")]
fn validate_serialsh_options(opts: &RunOptions) -> Result<()> {
    if matches!(opts.mode, RunMode::SerialShell) && (opts.payload_file.is_some() || opts.demo) {
        return Err(Error::InvalidArgs(
            "--serialsh cannot be combined with --demo or --payload-file".to_string(),
        ));
    }
    Ok(())
}

#[cfg(not(feature = "serialsh-preview"))]
fn validate_serialsh_options(_opts: &RunOptions) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_with_no_args() {
        let args: Vec<String> = vec![];
        let cmd = Command::parse(&args).unwrap();
        assert_eq!(cmd, Command::Run(RunOptions::default()));
    }

    #[test]
    fn parse_run_with_overrides() {
        let args = vec![
            "--device".into(),
            "/dev/ttyUSB0".into(),
            "--baud".into(),
            "9600".into(),
            "--cols".into(),
            "16".into(),
            "--rows".into(),
            "2".into(),
            "--payload-file".into(),
            "/tmp/payload.json".into(),
            "--backoff-initial-ms".into(),
            "750".into(),
            "--backoff-max-ms".into(),
            "9000".into(),
            "--pcf8574-addr".into(),
            "0x23".into(),
            "--log-level".into(),
            "debug".into(),
            "--log-file".into(),
            "/tmp/lifelinetty.log".into(),
            "--demo".into(),
        ];
        let expected = RunOptions {
            mode: RunMode::Daemon,
            device: Some("/dev/ttyUSB0".into()),
            baud: Some(9600),
            cols: Some(16),
            rows: Some(2),
            payload_file: Some("/tmp/payload.json".into()),
            backoff_initial_ms: Some(750),
            backoff_max_ms: Some(9000),
            pcf8574_addr: Some(Pcf8574Addr::Addr(0x23)),
            log_level: Some("debug".into()),
            log_file: Some("/tmp/lifelinetty.log".into()),
            demo: true,
        };
        let cmd = Command::parse(&args).unwrap();
        assert_eq!(cmd, Command::Run(expected));
    }

    #[test]
    fn parse_run_allows_implicit_subcommand() {
        let args = vec![
            "--device".into(),
            "/dev/ttyS1".into(),
            "--payload-file".into(),
            "/tmp/payload.json".into(),
        ];
        let expected = RunOptions {
            mode: RunMode::Daemon,
            device: Some("/dev/ttyS1".into()),
            baud: None,
            cols: None,
            rows: None,
            payload_file: Some("/tmp/payload.json".into()),
            backoff_initial_ms: None,
            backoff_max_ms: None,
            pcf8574_addr: None,
            log_level: None,
            log_file: None,
            demo: false,
        };
        let cmd = Command::parse(&args).unwrap();
        assert_eq!(cmd, Command::Run(expected));
    }

    #[test]
    fn parse_help() {
        let args = vec!["--help".into()];
        let cmd = Command::parse(&args).unwrap();
        assert_eq!(cmd, Command::ShowHelp);
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let args = vec!["--nope".into()];
        let err = Command::parse(&args).unwrap_err();
        assert!(format!("{err}").contains("unknown flag"));
    }

    #[cfg(feature = "serialsh-preview")]
    #[test]
    fn parse_serialsh_flag_sets_mode() {
        let args = vec!["--serialsh".into(), "--device".into(), "fake".into()];
        let cmd = Command::parse(&args).unwrap();
        match cmd {
            Command::Run(opts) => assert!(matches!(opts.mode, RunMode::SerialShell)),
            other => panic!("expected Run variant, got {other:?}"),
        }
    }

    #[cfg(feature = "serialsh-preview")]
    #[test]
    fn serialsh_disallows_demo_and_payload_file() {
        let args = vec!["--serialsh".into(), "--demo".into()];
        let err = Command::parse(&args).unwrap_err();
        assert!(format!("{err}").contains("serialsh"));

        let args = vec![
            "--serialsh".into(),
            "--payload-file".into(),
            "payload.json".into(),
        ];
        let err = Command::parse(&args).unwrap_err();
        assert!(format!("{err}").contains("serialsh"));
    }

    #[cfg(not(feature = "serialsh-preview"))]
    #[test]
    fn serialsh_flag_requires_feature() {
        let args = vec!["--serialsh".into()];
        let err = Command::parse(&args).unwrap_err();
        assert!(format!("{err}").contains("serialsh"));
    }
}
