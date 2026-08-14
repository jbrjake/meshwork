//! Declarative verify grammar (mw-sascrgs; DESIGN §12b, MW-E5): the
//! parse-only stage of retiring raw `sh -c`. A `verify:` string that
//! leads with a DSL keyword parses into predicates; parsing gives shell
//! metacharacters no meaning at all — they are just characters that fail
//! a class. Keyword-led text that does not parse REFUSES (`Malformed`) —
//! it never silently downgrades to shell, because that downgrade would
//! reopen exactly the drive-by hole the DSL closes. Everything else is
//! `LegacyShell` for the per-clone trust gate. No execution lives here.
//!
//! Grammar:
//! ```text
//! verify   := predicate | "all(" predicate ("," predicate)* ")"
//! predicate:= "exists" path | "absent" path
//!           | "contains" path (literal | "/" regex "/")
//!           | "run" runner-argv
//! ```
//! Paths are repo-relative: no leading `/` or `-`, no `..` segment.
//! Runner argvs are per-runner grammars, not an argv[0] allowlist
//! (Cursor GHSA-hf2x-r83r-qw5q / Flowise: allowlists fall to argument
//! injection) — today `cargo test|build|fmt`, every arg in a tight
//! character class with no leading dash. Regex patterns are stored raw,
//! delimiter-checked only; they may not contain `,` inside `all(…)`.

/// What a `verify:` string turned out to be.
pub enum Classified {
    /// Parsed predicates — safe by construction, no shell involved.
    Dsl(Vec<Predicate>),
    /// Keyword-led but invalid, with the reason: refuse loudly. Never
    /// run, never treated as shell.
    Malformed(String),
    /// Not DSL-shaped: legacy shell text for the MW-E5 trust gate.
    LegacyShell,
}

/// One declarative check.
pub enum Predicate {
    /// The repo-relative path names an existing file.
    Exists {
        /// Repo-relative path.
        path: String,
    },
    /// The repo-relative path names nothing.
    Absent {
        /// Repo-relative path.
        path: String,
    },
    /// The file's content matches the pattern.
    Contains {
        /// Repo-relative path.
        path: String,
        /// Literal token or raw regex.
        pattern: Pattern,
    },
    /// A known runner with a class-checked argv.
    Run {
        /// Full argv, runner first — executed argv-style, never a shell.
        argv: Vec<String>,
    },
}

/// A `contains` pattern.
pub enum Pattern {
    /// Fixed-string match, single tight-class token.
    Literal(String),
    /// Raw regex source (between `/` delimiters); compiled at execution,
    /// validated here only for delimiters.
    Regex(String),
}

/// Per-runner argv grammars: runner → allowed first args (subcommands).
/// Everything after the subcommand is a tight-class arg.
const RUNNERS: &[(&str, &[&str])] = &[("cargo", &["test", "build", "fmt"])];

const KEYWORDS: &[&str] = &["exists", "absent", "contains", "run"];

/// Classify one `verify:` string. Parsing only — nothing here executes.
#[must_use]
pub fn classify(text: &str) -> Classified {
    let t = text.trim();
    if let Some(inner) = t.strip_prefix("all(") {
        let Some(inner) = inner.strip_suffix(')') else {
            return Classified::Malformed("all( without closing )".into());
        };
        let mut preds = Vec::new();
        for part in inner.split(',') {
            match predicate(part.trim()) {
                Ok(p) => preds.push(p),
                Err(e) => return Classified::Malformed(e),
            }
        }
        return Classified::Dsl(preds);
    }
    let first = t.split_whitespace().next().unwrap_or("");
    if !KEYWORDS.contains(&first) {
        return Classified::LegacyShell;
    }
    match predicate(t) {
        Ok(p) => Classified::Dsl(vec![p]),
        Err(e) => Classified::Malformed(e),
    }
}

fn predicate(p: &str) -> Result<Predicate, String> {
    let (kw, rest) = p.split_once(char::is_whitespace).unwrap_or((p, ""));
    let rest = rest.trim();
    match kw {
        "exists" => one_path(kw, rest).map(|path| Predicate::Exists { path }),
        "absent" => one_path(kw, rest).map(|path| Predicate::Absent { path }),
        "contains" => {
            let (path_tok, pat) = rest
                .split_once(char::is_whitespace)
                .ok_or("contains needs <path> <literal|/regex/>")?;
            Ok(Predicate::Contains {
                path: path_token(path_tok)?,
                pattern: pattern(pat.trim())?,
            })
        }
        "run" => run_argv(rest).map(|argv| Predicate::Run { argv }),
        "" => Err("empty predicate".into()),
        other => Err(format!("unknown predicate: {other}")),
    }
}

fn one_path(kw: &str, rest: &str) -> Result<String, String> {
    if rest.is_empty() {
        return Err(format!("{kw} needs a path"));
    }
    if rest.split_whitespace().nth(1).is_some() {
        return Err(format!("{kw} takes exactly one path"));
    }
    path_token(rest)
}

/// Repo-relative, dash-free, traversal-free, tight class.
fn path_token(t: &str) -> Result<String, String> {
    if !class_ok(t, "_./-") {
        return Err(format!("bad path token: {t}"));
    }
    if t.starts_with('/') || t.starts_with('-') {
        return Err(format!("path must be repo-relative, dash-free: {t}"));
    }
    if t.split('/').any(|seg| seg == "..") {
        return Err(format!("path may not traverse up: {t}"));
    }
    Ok(t.to_string())
}

fn pattern(pat: &str) -> Result<Pattern, String> {
    if let Some(inner) = pat.strip_prefix('/') {
        let Some(inner) = inner.strip_suffix('/') else {
            return Err(format!("regex needs both / delimiters: {pat}"));
        };
        if inner.is_empty() {
            return Err("empty regex".into());
        }
        return Ok(Pattern::Regex(inner.to_string()));
    }
    if pat.starts_with('-') || !class_ok(pat, "_.:/=-") {
        return Err(format!("bad literal token: {pat} (use /regex/ for more)"));
    }
    Ok(Pattern::Literal(pat.to_string()))
}

/// Per-runner grammar: known runner, known subcommand, tight-class args
/// with no leading dash — flags are the injection surface, so no flags.
fn run_argv(rest: &str) -> Result<Vec<String>, String> {
    let mut toks = rest.split_whitespace();
    let runner = toks.next().ok_or("run needs <runner> <subcommand> …")?;
    let subs = RUNNERS
        .iter()
        .find(|(r, _)| *r == runner)
        .map(|(_, subs)| *subs)
        .ok_or_else(|| format!("unknown runner: {runner}"))?;
    let sub = toks
        .next()
        .ok_or_else(|| format!("{runner} needs a subcommand"))?;
    if !subs.contains(&sub) {
        return Err(format!("{runner} {sub} is not in the runner grammar"));
    }
    let mut argv = vec![runner.to_string(), sub.to_string()];
    for arg in toks {
        if arg.starts_with('-') || !class_ok(arg, "_.:/=-") {
            return Err(format!("bad arg token: {arg}"));
        }
        argv.push(arg.to_string());
    }
    Ok(argv)
}

/// ASCII-alphanumeric plus `extra`, non-empty.
fn class_ok(t: &str, extra: &str) -> bool {
    !t.is_empty()
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || extra.contains(c))
}

impl std::fmt::Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::Exists { path } => write!(f, "exists {path}"),
            Predicate::Absent { path } => write!(f, "absent {path}"),
            Predicate::Contains { path, pattern } => write!(f, "contains {path} {pattern}"),
            Predicate::Run { argv } => write!(f, "run {}", argv.join(" ")),
        }
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Literal(l) => write!(f, "{l}"),
            Pattern::Regex(r) => write!(f, "/{r}/"),
        }
    }
}
