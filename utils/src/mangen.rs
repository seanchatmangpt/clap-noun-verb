use clap::Command;
use std::io::Write;

/// Generate troff man pages using `clap_mangen`.
///
/// # Examples
///
/// ```
/// use clap::Command;
/// use clap_noun_verb_utils::mangen::generate_manpage;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let cmd = Command::new("myapp");
/// let mut buffer = Vec::new();
/// generate_manpage(&cmd, &mut buffer)?;
///
/// let output = String::from_utf8(buffer)?;
/// assert!(output.contains(".TH"));
/// assert!(output.to_uppercase().contains("MYAPP"));
/// # Ok(())
/// # }
/// ```
pub fn generate_manpage(cmd: &Command, buf: &mut dyn Write) -> std::io::Result<()> {
    let man = clap_mangen::Man::new(cmd.clone());
    man.render(buf)
}
