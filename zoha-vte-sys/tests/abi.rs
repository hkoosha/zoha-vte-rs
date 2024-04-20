// Generated by gir (https://github.com/gtk-rs/gir @ bd704e8f22c9)
// from .. (@ 32b461bf5f55+)
// from ../gir-files (@ 20031a537e40)
// DO NOT EDIT

#![cfg(unix)]

use zoha_vte_sys::*;
use std::mem::{align_of, size_of};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use tempfile::Builder;

static PACKAGES: &[&str] = &["vte-2.91"];

#[derive(Clone, Debug)]
struct Compiler {
    pub args: Vec<String>,
}

impl Compiler {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut args = get_var("CC", "cc")?;
        args.push("-Wno-deprecated-declarations".to_owned());
        // For _Generic
        args.push("-std=c11".to_owned());
        // For %z support in printf when using MinGW.
        args.push("-D__USE_MINGW_ANSI_STDIO".to_owned());
        args.extend(get_var("CFLAGS", "")?);
        args.extend(get_var("CPPFLAGS", "")?);
        args.extend(pkg_config_cflags(PACKAGES)?);
        Ok(Self { args })
    }

    pub fn compile(&self, src: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
        let mut cmd = self.to_command();
        cmd.arg(src);
        cmd.arg("-o");
        cmd.arg(out);
        let status = cmd.spawn()?.wait()?;
        if !status.success() {
            return Err(format!("compilation command {cmd:?} failed, {status}").into());
        }
        Ok(())
    }

    fn to_command(&self) -> Command {
        let mut cmd = Command::new(&self.args[0]);
        cmd.args(&self.args[1..]);
        cmd
    }
}

fn get_var(name: &str, default: &str) -> Result<Vec<String>, Box<dyn Error>> {
    match env::var(name) {
        Ok(value) => Ok(shell_words::split(&value)?),
        Err(env::VarError::NotPresent) => Ok(shell_words::split(default)?),
        Err(err) => Err(format!("{name} {err}").into()),
    }
}

fn pkg_config_cflags(packages: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    let pkg_config = env::var_os("PKG_CONFIG")
        .unwrap_or_else(|| OsString::from("pkg-config"));
    let mut cmd = Command::new(pkg_config);
    cmd.arg("--cflags");
    cmd.args(packages);
    cmd.stderr(Stdio::inherit());
    let out = cmd.output()?;
    if !out.status.success() {
        let (status, stdout) = (out.status, String::from_utf8_lossy(&out.stdout));
        return Err(format!("command {cmd:?} failed, {status:?}\nstdout: {stdout}").into());
    }
    let stdout = str::from_utf8(&out.stdout)?;
    Ok(shell_words::split(stdout.trim())?)
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Layout {
    size: usize,
    alignment: usize,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
struct Results {
    /// Number of successfully completed tests.
    passed: usize,
    /// Total number of failed tests (including those that failed to compile).
    failed: usize,
}

impl Results {
    fn record_passed(&mut self) {
        self.passed += 1;
    }
    fn record_failed(&mut self) {
        self.failed += 1;
    }
    fn summary(&self) -> String {
        format!("{} passed; {} failed", self.passed, self.failed)
    }
    fn expect_total_success(&self) {
        if self.failed == 0 {
            println!("OK: {}", self.summary());
        } else {
            panic!("FAILED: {}", self.summary());
        };
    }
}

#[test]
fn cross_validate_constants_with_c() {
    let mut c_constants: Vec<(String, String)> = Vec::new();

    for l in get_c_output("constant").unwrap().lines() {
        let (name, value) = l.split_once(';').expect("Missing ';' separator");
        c_constants.push((name.to_owned(), value.to_owned()));
    }

    let mut results = Results::default();

    for ((rust_name, rust_value), (c_name, c_value)) in
        RUST_CONSTANTS.iter().zip(c_constants.iter())
    {
        if rust_name != c_name {
            results.record_failed();
            eprintln!("Name mismatch:\nRust: {rust_name:?}\nC:    {c_name:?}");
            continue;
        }

        if rust_value != c_value {
            results.record_failed();
            eprintln!(
                "Constant value mismatch for {rust_name}\nRust: {rust_value:?}\nC:    {c_value:?}",
            );
            continue;
        }

        results.record_passed();
    }

    results.expect_total_success();
}

#[test]
fn cross_validate_layout_with_c() {
    let mut c_layouts = Vec::new();

    for l in get_c_output("layout").unwrap().lines() {
        let (name, value) = l.split_once(';').expect("Missing first ';' separator");
        let (size, alignment) = value.split_once(';').expect("Missing second ';' separator");
        let size = size.parse().expect("Failed to parse size");
        let alignment = alignment.parse().expect("Failed to parse alignment");
        c_layouts.push((name.to_owned(), Layout { size, alignment }));
    }

    let mut results = Results::default();

    for ((rust_name, rust_layout), (c_name, c_layout)) in
        RUST_LAYOUTS.iter().zip(c_layouts.iter())
    {
        if rust_name != c_name {
            results.record_failed();
            eprintln!("Name mismatch:\nRust: {rust_name:?}\nC:    {c_name:?}");
            continue;
        }

        if rust_layout != c_layout {
            results.record_failed();
            eprintln!(
                "Layout mismatch for {rust_name}\nRust: {rust_layout:?}\nC:    {c_layout:?}",
            );
            continue;
        }

        results.record_passed();
    }

    results.expect_total_success();
}

fn get_c_output(name: &str) -> Result<String, Box<dyn Error>> {
    let tmpdir = Builder::new().prefix("abi").tempdir()?;
    let exe = tmpdir.path().join(name);
    let c_file = Path::new("tests").join(name).with_extension("c");

    let cc = Compiler::new().expect("configured compiler");
    cc.compile(&c_file, &exe)?;

    let mut cmd = Command::new(exe);
    cmd.stderr(Stdio::inherit());
    let out = cmd.output()?;
    if !out.status.success() {
        let (status, stdout) = (out.status, String::from_utf8_lossy(&out.stdout));
        return Err(format!("command {cmd:?} failed, {status:?}\nstdout: {stdout}").into());
    }

    Ok(String::from_utf8(out.stdout)?)
}

const RUST_LAYOUTS: &[(&str, Layout)] = &[
    ("VteAlign", Layout {size: size_of::<VteAlign>(), alignment: align_of::<VteAlign>()}),
    ("VteCursorBlinkMode", Layout {size: size_of::<VteCursorBlinkMode>(), alignment: align_of::<VteCursorBlinkMode>()}),
    ("VteCursorShape", Layout {size: size_of::<VteCursorShape>(), alignment: align_of::<VteCursorShape>()}),
    ("VteEraseBinding", Layout {size: size_of::<VteEraseBinding>(), alignment: align_of::<VteEraseBinding>()}),
    ("VteFeatureFlags", Layout {size: size_of::<VteFeatureFlags>(), alignment: align_of::<VteFeatureFlags>()}),
    ("VteFormat", Layout {size: size_of::<VteFormat>(), alignment: align_of::<VteFormat>()}),
    ("VtePtyError", Layout {size: size_of::<VtePtyError>(), alignment: align_of::<VtePtyError>()}),
    ("VtePtyFlags", Layout {size: size_of::<VtePtyFlags>(), alignment: align_of::<VtePtyFlags>()}),
    ("VteRegexError", Layout {size: size_of::<VteRegexError>(), alignment: align_of::<VteRegexError>()}),
    ("VteTerminal", Layout {size: size_of::<VteTerminal>(), alignment: align_of::<VteTerminal>()}),
    ("VteTerminalClass", Layout {size: size_of::<VteTerminalClass>(), alignment: align_of::<VteTerminalClass>()}),
    ("VteTextBlinkMode", Layout {size: size_of::<VteTextBlinkMode>(), alignment: align_of::<VteTextBlinkMode>()}),
    ("VteWriteFlags", Layout {size: size_of::<VteWriteFlags>(), alignment: align_of::<VteWriteFlags>()}),
];

const RUST_CONSTANTS: &[(&str, &str)] = &[
    ("(gint) VTE_ALIGN_CENTER", "1"),
    ("(gint) VTE_ALIGN_END", "3"),
    ("(gint) VTE_ALIGN_START", "0"),
    ("(gint) VTE_CURSOR_BLINK_OFF", "2"),
    ("(gint) VTE_CURSOR_BLINK_ON", "1"),
    ("(gint) VTE_CURSOR_BLINK_SYSTEM", "0"),
    ("(gint) VTE_CURSOR_SHAPE_BLOCK", "0"),
    ("(gint) VTE_CURSOR_SHAPE_IBEAM", "1"),
    ("(gint) VTE_CURSOR_SHAPE_UNDERLINE", "2"),
    ("(gint) VTE_ERASE_ASCII_BACKSPACE", "1"),
    ("(gint) VTE_ERASE_ASCII_DELETE", "2"),
    ("(gint) VTE_ERASE_AUTO", "0"),
    ("(gint) VTE_ERASE_DELETE_SEQUENCE", "3"),
    ("(gint) VTE_ERASE_TTY", "4"),
    ("(guint) VTE_FEATURE_FLAGS_MASK", "4294967295"),
    ("(guint) VTE_FEATURE_FLAG_BIDI", "1"),
    ("(guint) VTE_FEATURE_FLAG_ICU", "2"),
    ("(guint) VTE_FEATURE_FLAG_SIXEL", "8"),
    ("(guint) VTE_FEATURE_FLAG_SYSTEMD", "4"),
    ("(gint) VTE_FORMAT_HTML", "2"),
    ("(gint) VTE_FORMAT_TEXT", "1"),
    ("VTE_MAJOR_VERSION", "0"),
    ("VTE_MICRO_VERSION", "0"),
    ("VTE_MINOR_VERSION", "75"),
    ("(guint) VTE_PTY_DEFAULT", "0"),
    ("(gint) VTE_PTY_ERROR_PTY98_FAILED", "1"),
    ("(gint) VTE_PTY_ERROR_PTY_HELPER_FAILED", "0"),
    ("(guint) VTE_PTY_NO_CTTY", "64"),
    ("(guint) VTE_PTY_NO_FALLBACK", "16"),
    ("(guint) VTE_PTY_NO_HELPER", "8"),
    ("(guint) VTE_PTY_NO_LASTLOG", "1"),
    ("(guint) VTE_PTY_NO_SESSION", "32"),
    ("(guint) VTE_PTY_NO_UTMP", "2"),
    ("(guint) VTE_PTY_NO_WTMP", "4"),
    ("(gint) VTE_REGEX_ERROR_INCOMPATIBLE", "2147483646"),
    ("(gint) VTE_REGEX_ERROR_NOT_SUPPORTED", "2147483647"),
    ("VTE_REGEX_FLAGS_DEFAULT", "1075314688"),
    ("VTE_SPAWN_NO_PARENT_ENVV", "33554432"),
    ("VTE_SPAWN_NO_SYSTEMD_SCOPE", "67108864"),
    ("VTE_SPAWN_REQUIRE_SYSTEMD_SCOPE", "134217728"),
    ("VTE_TEST_FLAGS_ALL", "18446744073709551615"),
    ("VTE_TEST_FLAGS_NONE", "0"),
    ("(gint) VTE_TEXT_BLINK_ALWAYS", "3"),
    ("(gint) VTE_TEXT_BLINK_FOCUSED", "1"),
    ("(gint) VTE_TEXT_BLINK_NEVER", "0"),
    ("(gint) VTE_TEXT_BLINK_UNFOCUSED", "2"),
    ("(gint) VTE_WRITE_DEFAULT", "0"),
];


