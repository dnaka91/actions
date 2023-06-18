use std::{
    env,
    io::BufReader,
    process::{Command, Stdio},
};

use anyhow::{ensure, Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::Message;
use heck::{ToShoutySnakeCase, ToSnakeCase};
use target_lexicon::Triple;

pub struct CargoBuilder<'a> {
    cmd: Command,
    bin: &'a str,
    target: String,
}

impl<'a> CargoBuilder<'a> {
    #[must_use]
    pub fn new(bin: &'a str, target: &'a Triple) -> Self {
        let target = target.to_string();

        let rustflags = match env::var_os("RUSTFLAGS") {
            Some(flags) => format!("{} -C strip=symbols", flags.to_string_lossy()),
            None => "-C strip=symbols".to_owned(),
        };

        let mut cmd = Command::new("cargo");
        cmd.args(["build", "--release"])
            .args(["--message-format", "json-render-diagnostics"])
            .args(["--bin", bin])
            .args(["--target", &target])
            .env("RUSTFLAGS", rustflags)
            .stdout(Stdio::piped());

        Self { cmd, bin, target }
    }

    #[must_use]
    pub fn with_compiler(mut self, compiler: Option<&str>) -> Self {
        if let Some(compiler) = compiler {
            self.cmd.envs([
                (format!("CC_{}", self.target.to_snake_case()), compiler),
                (
                    format!("CARGO_TARGET_{}_LINKER", self.target.to_shouty_snake_case()),
                    compiler,
                ),
            ]);
        }

        self
    }

    #[must_use]
    pub fn with_features(mut self, features: &[String]) -> Self {
        if !features.is_empty() {
            self.cmd.arg("--features").arg(features.join(","));
        }

        self
    }

    pub fn run(mut self) -> Result<Utf8PathBuf> {
        let mut child = self.cmd.spawn()?;
        let reader = BufReader::new(child.stdout.take().unwrap());
        let mut binary = None;

        for msg in Message::parse_stream(reader) {
            match msg? {
                Message::CompilerArtifact(artifact) => {
                    if let Some(bin) = artifact.executable.filter(|_| {
                        artifact.target.name == self.bin && artifact.target.kind == ["bin"]
                    }) {
                        binary = binary.or(Some(bin));
                    }
                }
                Message::CompilerMessage(message) => println!("{message}"),
                Message::BuildFinished(build) => {
                    ensure!(build.success, "cargo build failed");
                }
                Message::TextLine(line) => println!("{line}"),
                _ => {}
            }
        }

        binary.context("failed finding binary after compilation")
    }
}
