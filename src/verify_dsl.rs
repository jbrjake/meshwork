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
//! predicate:= "exists" path-or-glob | "absent" path
//!           | "contains" path (literal | "/" regex "/")
//!           | "lacks" path (literal | "/" regex "/")
//!           | "run" runner-argv
//! ```
//! Paths are repo-relative: no leading `/` or `-`, no `..` segment; an
//! `exists` path may carry one `*` inside its last segment (MW-N4).
//! Runner argvs are per-runner grammars, not an argv[0] allowlist
//! (Cursor GHSA-hf2x-r83r-qw5q / Flowise: allowlists fall to argument
//! injection) — today `cargo test|build|fmt`, every arg in a tight
//! character class with no leading dash; `package=<crate>` and
//! `target=<name>` are the dash-free spellings of `-p`/`--test`, which
//! the executor — never this parser — writes into the argv (MW-N1/N2).
//! Regex patterns are stored raw, delimiter-checked only; they may not
//! contain `,` inside `all(…)`.

/// The grammar as help text — printed by `verify --help` and
/// `close --help`, so the shapes that close accepts are one `--help`
/// away instead of a defect report (mw-8e769q0).
pub const GRAMMAR_HELP: &str = "Verify grammar (the close gate; one predicate, or all(p, p, ...)):
  exists <path>                the repo-relative path exists (one * allowed in the last segment)
  absent <path>                the path does not exist
  contains <path> <token>      the file contains the literal token
  contains <path> /<regex>/    the file matches the regex (grep-like: ^ $ anchor lines;
                               lead with (?s) to let . cross a line wrap)
  lacks <path> <token|/regex/> the file exists and does not match — a missing file refuses
  run cargo test|build|fmt <args...>   spawned argv-style, never a shell; args carry
                               no leading dash (letters, digits, _ . : / = -);
                               package=<crate> and target=<name> spell -p and --test
  all(<pred>, <pred>, ...)     every predicate must hold
Paths: no leading / or -, no .. segment. run cargo test must observe `ok. N passed`, N >= 1.
Anything not keyword-led is legacy shell: it runs only behind the per-clone approval gate.";

/// One line naming the grammar — for refusals at authoring time.
pub const GRAMMAR_LINE: &str = "exists|absent <path> · contains|lacks <path> <token|/regex/> · \
                                run cargo test|build|fmt <args> [package=<crate> target=<name>] \
                                · all(p, p)";

/// The refusal for a keyword-led verify that does not parse — the same
/// words at `add`, `set`, `add --batch` and `start`, so a verify close
/// would refuse is never minted (mw-8e769q0).
#[must_use]
pub fn malformed_refusal(verify: &str, why: &str) -> String {
    format!(
        "verify `{verify}` is keyword-led but does not parse ({why}) — close would refuse it; \
         grammar: {GRAMMAR_LINE} (verify --help)"
    )
}

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
    /// The file exists and its content does NOT match the pattern (MW-N3);
    /// a missing file refuses rather than passing.
    Lacks {
        /// Repo-relative path.
        path: String,
        /// Literal token or raw regex.
        pattern: Pattern,
    },
    /// A known runner with a class-checked argv.
    Run {
        /// Full argv, runner first, tokens as written — the executor spells
        /// `package=`/`target=` as `-p`/`--test` when it spawns.
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

const KEYWORDS: &[&str] = &["exists", "absent", "contains", "lacks", "run"];

/// Dash-free spellings of the two flags a scoped `cargo test` needs
/// (MW-N1): token prefix → the flag the executor writes. A typed flag
/// refuses naming its spelling (MW-N2).
pub const SCOPING_TOKENS: &[(&str, &str)] = &[("package=", "-p"), ("target=", "--test")];

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
        "exists" => one_path(kw, rest, "_./-*")
            .and_then(|path| glob_path(&path))
            .map(|path| Predicate::Exists { path }),
        "absent" => one_path(kw, rest, "_./-").map(|path| Predicate::Absent { path }),
        "contains" | "lacks" => {
            let (path_tok, pat) = rest
                .split_once(char::is_whitespace)
                .ok_or_else(|| format!("{kw} needs <path> <literal|/regex/>"))?;
            let path = file_path_token(kw, path_tok)?;
            let pattern = pattern(pat.trim())?;
            Ok(if kw == "lacks" {
                Predicate::Lacks { path, pattern }
            } else {
                Predicate::Contains { path, pattern }
            })
        }
        "run" => run_argv(rest).map(|argv| Predicate::Run { argv }),
        "" => Err("empty predicate".into()),
        other => Err(format!("unknown predicate: {other}")),
    }
}

/// A path `exists` may glob: at most one `*`, inside the last segment,
/// beside at least one literal character (MW-N4 — one dated artifact,
/// never a directory walk). Plain paths pass through.
fn glob_path(path: &str) -> Result<String, String> {
    let stars = path.matches('*').count();
    if stars == 0 {
        return Ok(path.to_string());
    }
    let last = path.rsplit('/').next().unwrap_or(path);
    if stars > 1 || !last.contains('*') || last == "*" {
        return Err(format!(
            "exists takes one * inside the last path segment, beside literal text: {path}"
        ));
    }
    Ok(path.to_string())
}

/// A path `contains`/`lacks` reads: a file, never a directory — the
/// trailing slash that would ask for a walk refuses at parse (MW-N4).
fn file_path_token(kw: &str, t: &str) -> Result<String, String> {
    if t.ends_with('/') {
        return Err(format!("{kw} reads one file, never a directory: {t}"));
    }
    path_token(t)
}

fn one_path(kw: &str, rest: &str, class: &str) -> Result<String, String> {
    if rest.is_empty() {
        return Err(format!("{kw} needs a path"));
    }
    if rest.split_whitespace().nth(1).is_some() {
        return Err(format!("{kw} takes exactly one path"));
    }
    path_token_in(rest, class)
}

/// Repo-relative, dash-free, traversal-free, tight class.
fn path_token(t: &str) -> Result<String, String> {
    path_token_in(t, "_./-")
}

fn path_token_in(t: &str, class: &str) -> Result<String, String> {
    if !class_ok(t, class) {
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
        if arg.starts_with('-') {
            return Err(dash_refusal(arg));
        }
        if !class_ok(arg, "_.:/=-") {
            return Err(format!("bad arg token: {arg}"));
        }
        scoping_token_ok(sub, arg)?;
        argv.push(arg.to_string());
    }
    Ok(argv)
}

/// A dash-led arg refuses (flags are the injection surface); the refusal
/// names the dash-free spelling where one exists (MW-N2).
fn dash_refusal(arg: &str) -> String {
    let spelled = match arg {
        "-p" | "--package" => Some("package=<crate>"),
        "--test" => Some("target=<name>"),
        _ => None,
    };
    match spelled {
        Some(s) => format!("bad arg token: {arg} — args carry no leading dash; spell it {s}"),
        None => format!("bad arg token: {arg} — args carry no leading dash"),
    }
}

/// `package=<crate>` is legal on `test` and `build`, `target=<name>` on
/// `test`; each value is one crate-or-target name (letters, digits, `_`,
/// `-`), never empty, never another `=` (MW-N1).
fn scoping_token_ok(sub: &str, arg: &str) -> Result<(), String> {
    for (prefix, flag) in SCOPING_TOKENS {
        let Some(value) = arg.strip_prefix(prefix) else {
            continue;
        };
        let allowed = match *flag {
            "-p" => matches!(sub, "test" | "build"),
            _ => sub == "test",
        };
        if !allowed {
            return Err(format!("{prefix}<name> is not a cargo {sub} token"));
        }
        if !class_ok(value, "_-") {
            return Err(format!(
                "bad {prefix} value: {value:?} — one crate or target name (letters, digits, _ -)"
            ));
        }
    }
    Ok(())
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
            Predicate::Lacks { path, pattern } => write!(f, "lacks {path} {pattern}"),
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
