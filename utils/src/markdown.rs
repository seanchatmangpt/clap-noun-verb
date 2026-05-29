use clap::Command;
use std::io::Write;

/// Generate clean, comprehensive markdown help documentation by recursively walking the `clap::Command` tree.
pub fn generate_markdown(cmd: &Command, buf: &mut dyn Write) -> std::io::Result<()> {
    write_markdown_recursive(cmd, buf, 1)
}

fn write_markdown_recursive(
    cmd: &Command,
    buf: &mut dyn Write,
    depth: usize,
) -> std::io::Result<()> {
    let title_prefix = "#".repeat(depth);
    writeln!(buf, "{} {}", title_prefix, cmd.get_name())?;
    writeln!(buf)?;

    if let Some(about) = cmd.get_about() {
        writeln!(buf, "{}", about)?;
        writeln!(buf)?;
    }

    if let Some(version) = cmd.get_version() {
        writeln!(buf, "**Version:** {}", version)?;
        writeln!(buf)?;
    }

    writeln!(buf, "{} Usage", title_prefix)?;
    writeln!(buf)?;
    writeln!(buf, "```")?;
    write!(buf, "{}", cmd.get_name())?;

    let mut has_options = false;
    let mut has_positional = false;
    for arg in cmd.get_arguments() {
        if arg.is_positional() {
            has_positional = true;
        } else {
            has_options = true;
        }
    }

    if has_options {
        write!(buf, " [OPTIONS]")?;
    }

    if has_positional {
        for arg in cmd.get_arguments().filter(|a| a.is_positional()) {
            let name = arg.get_id().as_str();
            if arg.is_required_set() {
                write!(buf, " <{}>", name)?;
            } else {
                write!(buf, " [{}]", name)?;
            }
        }
    }
    writeln!(buf)?;
    writeln!(buf, "```")?;
    writeln!(buf)?;

    let args: Vec<_> = cmd.get_arguments().collect();
    if !args.is_empty() {
        writeln!(buf, "{} Arguments / Options", title_prefix)?;
        writeln!(buf)?;
        for arg in args {
            let mut arg_str = String::new();
            if let Some(short) = arg.get_short() {
                arg_str.push_str(&format!("`-{}`", short));
            }
            if let Some(long) = arg.get_long() {
                if !arg_str.is_empty() {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&format!("`--{}`", long));
            }
            if arg_str.is_empty() {
                arg_str.push_str(&format!("`<{}>`", arg.get_id().as_str()));
            }

            writeln!(buf, "- **{}**", arg_str)?;
            if let Some(help) = arg.get_help() {
                writeln!(buf, "  - {}", help)?;
            }
            if let Some(env) = arg.get_env() {
                if let Some(env_str) = env.to_str() {
                    writeln!(buf, "  - Environment Variable: `{}`", env_str)?;
                }
            }
            let is_req = arg.is_required_set();
            writeln!(buf, "  - Required: `{}`", is_req)?;
        }
        writeln!(buf)?;
    }

    let subcommands: Vec<_> = cmd.get_subcommands().filter(|c| !c.is_hide_set()).collect();
    if !subcommands.is_empty() {
        writeln!(buf, "{} Subcommands", title_prefix)?;
        writeln!(buf)?;
        for sub in &subcommands {
            writeln!(buf, "- [`{}`](#{})", sub.get_name(), slugify(sub.get_name()))?;
        }
        writeln!(buf)?;

        for sub in subcommands {
            write_markdown_recursive(sub, buf, depth + 1)?;
        }
    }

    Ok(())
}

fn slugify(s: &str) -> String {
    let mut slug = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            slug.push(c.to_ascii_lowercase());
        } else if c == ' ' {
            slug.push('-');
        }
    }
    slug
}
