use anyhow::{Result, bail};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaunchOptions {
    pub automatic: bool,
    pub preview: bool,
    pub select_files: bool,
    pub choose_destination: bool,
    pub message: Option<String>,
}

pub fn parse_args(args: &[String]) -> Result<LaunchOptions> {
    if args.is_empty() {
        return Ok(LaunchOptions::default());
    }
    if !matches!(args[0].as_str(), "c" | "com" | "commit") {
        bail!(
            "unknown command `{}`; use `fgit`, `fgit c`, `fgit com`, or `fgit commit`",
            args[0]
        );
    }

    let mut options = LaunchOptions {
        automatic: true,
        ..LaunchOptions::default()
    };
    let mut index = 1;
    while index < args.len() {
        let argument = &args[index];
        if argument == "--message" || argument == "-m" {
            index += 1;
            let Some(message) = args.get(index) else {
                bail!("{argument} needs a message value");
            };
            options.message = Some(message.clone());
        } else if let Some(message) = argument.strip_prefix("--message=") {
            options.message = Some(message.to_string());
        } else if let Some(message) = argument.strip_prefix("-m") {
            if message.is_empty() {
                bail!("-m needs a message value");
            }
            options.message = Some(message.to_string());
        } else if argument.starts_with('-') && !argument.starts_with("--") {
            for flag in argument[1..].chars() {
                match flag {
                    'p' => options.preview = true,
                    's' => options.select_files = true,
                    'r' => options.choose_destination = true,
                    _ => bail!("unknown flag `-{flag}`; supported flags are -p, -s, -r, and -m"),
                }
            }
        } else {
            bail!("unexpected argument `{argument}`");
        }
        index += 1;
    }
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn parses_every_documented_fast_command_without_a_shell() {
        for alias in ["c", "com", "commit"] {
            assert!(parse_args(&args(&[alias])).unwrap().automatic);
        }
        let flags = parse_args(&args(&["c", "-psr"])).unwrap();
        assert!(flags.preview && flags.select_files && flags.choose_destination);
        assert_eq!(
            parse_args(&args(&["c", "-m", "fix: quoted value"]))
                .unwrap()
                .message
                .as_deref(),
            Some("fix: quoted value")
        );
        assert_eq!(
            parse_args(&args(&["c", "--message=fix: inline"]))
                .unwrap()
                .message
                .as_deref(),
            Some("fix: inline")
        );
    }

    #[test]
    fn rejects_unknown_or_incomplete_arguments() {
        assert!(parse_args(&args(&["status"])).is_err());
        assert!(parse_args(&args(&["c", "-x"])).is_err());
        assert!(parse_args(&args(&["c", "-m"])).is_err());
    }
}
