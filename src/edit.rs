//! Surgical text edits to task files: one-line frontmatter changes and
//! end-of-section appends. This is MW-I1's discipline made mechanical —
//! a status change must diff as one line so union merges stay clean.

use std::fmt::Write as _;

/// Split `text` into its frontmatter body and everything from the
/// closing fence on.
fn split_frontmatter(text: &str) -> Result<(&str, &str), String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Err("missing frontmatter fences".to_string());
    };
    let Some(end) = rest.find("\n---") else {
        return Err("missing closing frontmatter fence".to_string());
    };
    Ok((&rest[..end], &rest[end..]))
}

/// Index of the first frontmatter line after the block under the key at
/// `key_at`. Indented lines continue the block, and so do blank lines
/// followed by another indented line: a block scalar may carry its
/// paragraph breaks unindented, and the whole of it is one value. Blank
/// lines after the last indented line belong to whatever follows.
fn block_end(lines: &[&str], key_at: usize) -> usize {
    let mut end = key_at + 1;
    let mut i = key_at + 1;
    while i < lines.len() {
        if lines[i].starts_with(' ') {
            i += 1;
            end = i;
        } else if lines[i].trim().is_empty() {
            i += 1;
        } else {
            break;
        }
    }
    end
}

/// Rebuild the frontmatter with the first `key:` line and its block
/// replaced by `rendered` (nothing, when `rendered` is None); when the
/// key is absent, `absent` decides what — if anything — lands before the
/// closing fence.
fn replace_block(
    text: &str,
    key: &str,
    rendered: Option<&str>,
    absent: Option<&str>,
) -> Result<String, String> {
    let (fm, tail) = split_frontmatter(text)?;
    let prefix = format!("{key}:");
    let fm_lines: Vec<&str> = fm.lines().collect();
    let mut lines: Vec<&str> = Vec::new();
    let mut i = 0;
    let mut replaced = false;
    while i < fm_lines.len() {
        if !replaced && fm_lines[i].starts_with(&prefix) {
            lines.extend(rendered);
            replaced = true;
            i = block_end(&fm_lines, i);
        } else {
            lines.push(fm_lines[i]);
            i += 1;
        }
    }
    if !replaced {
        lines.extend(absent);
    }
    Ok(format!("---\n{}{tail}", lines.join("\n")))
}

/// Replace the first `key:` line inside the frontmatter with `key: value`
/// (or a bare `key:` when `value` is None); insert before the closing fence
/// when the key is absent. Everything else is preserved byte-for-byte.
///
/// # Errors
/// When the text has no frontmatter fences — that file is invalid and
/// should be repaired by lint, not edited blind.
pub fn set_scalar(text: &str, key: &str, value: Option<&str>) -> Result<String, String> {
    let (fm, tail) = split_frontmatter(text)?;
    let rendered = match value {
        Some(v) => format!("{key}: {v}"),
        None => format!("{key}:"),
    };
    let prefix = format!("{key}:");
    let mut lines: Vec<String> = Vec::new();
    let mut replaced = false;
    for line in fm.lines() {
        if !replaced && line.starts_with(&prefix) {
            lines.push(rendered.clone());
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(rendered);
    }
    Ok(format!("---\n{}{tail}", lines.join("\n")))
}

/// Remove the first `key:` line — and the block under it, which in
/// frontmatter YAML can only be that key's value — from the frontmatter
/// (used when a list empties — a bare `needs: []` is noise in a
/// hand-editable file — and when a terminal transition drops `handoff:`).
///
/// # Errors
/// When fences are missing, like [`set_scalar`].
pub fn remove_scalar(text: &str, key: &str) -> Result<String, String> {
    replace_block(text, key, None, None)
}

/// Replace `key:` and the block under it with an inline list
/// (`key: [a, b]`), inserting before the closing fence when absent. Items
/// pass through [`crate::write::yaml_scalar`].
///
/// # Errors
/// When fences are missing, like [`set_scalar`].
pub fn set_list(text: &str, key: &str, items: &[String]) -> Result<String, String> {
    let rendered = format!(
        "{key}: [{}]",
        items
            .iter()
            .map(|i| crate::write::yaml_scalar(i))
            .collect::<Vec<_>>()
            .join(", ")
    );
    replace_block(text, key, Some(&rendered), Some(&rendered))
}

/// Replace `key:` and the block under it with a literal block scalar
/// (`key: |` + two-space-indented lines), inserting before the closing
/// fence when the key is absent (mw-0f4j: `set --handoff`).
///
/// # Errors
/// When fences are missing, like [`set_scalar`].
pub fn set_block(text: &str, key: &str, block_lines: &[String]) -> Result<String, String> {
    let mut rendered = format!("{key}: |");
    for line in block_lines {
        rendered.push('\n');
        rendered.push_str("  ");
        rendered.push_str(line);
    }
    replace_block(text, key, Some(&rendered), Some(&rendered))
}

/// Append one `  - item` to the indented block list under `key:`, creating
/// the key when absent. Existing items — including their trailing `# …`
/// hand comments — are preserved byte-for-byte (mw-0f4j: `set --docs`).
///
/// # Errors
/// When fences are missing, like [`set_scalar`].
pub fn append_block_item(text: &str, key: &str, item: &str) -> Result<String, String> {
    let (fm, tail) = split_frontmatter(text)?;
    let prefix = format!("{key}:");
    let rendered = format!("  - {item}");
    let fm_lines: Vec<&str> = fm.lines().collect();
    let mut lines: Vec<&str> = Vec::new();
    let mut inserted = false;
    let mut i = 0;
    while i < fm_lines.len() {
        lines.push(fm_lines[i]);
        if !inserted && fm_lines[i].starts_with(&prefix) {
            // copy the existing block whole, then append after its last item
            let end = block_end(&fm_lines, i);
            lines.extend(&fm_lines[i + 1..end]);
            lines.push(&rendered);
            inserted = true;
            i = end;
            continue;
        }
        i += 1;
    }
    let created = format!("{key}:");
    if !inserted {
        lines.push(&created);
        lines.push(&rendered);
    }
    Ok(format!("---\n{}{tail}", lines.join("\n")))
}

/// Mend the frontmatter a narrower block strip left behind: indented
/// lines stranded after a blank line under a key that opened no block
/// (a plain `key: value`, or the fence itself). Returns the repaired text
/// and how many lines were dropped, or None when the shape is not this
/// one — anything else is not mechanical and stays for a human.
#[must_use]
pub fn strip_stranded_block(text: &str) -> Option<(String, usize)> {
    let (fm, tail) = split_frontmatter(text).ok()?;
    let fm_lines: Vec<&str> = fm.lines().collect();
    let mut kept: Vec<&str> = Vec::new();
    let mut dropped = 0;
    let mut owner_opens_block = false;
    let mut i = 0;
    while i < fm_lines.len() {
        let line = fm_lines[i];
        if line.trim().is_empty() {
            // A blank followed by indented lines under a non-block owner
            // is the stranded shape: drop the blank and the run.
            let mut j = i + 1;
            while j < fm_lines.len() && fm_lines[j].trim().is_empty() {
                j += 1;
            }
            let run_start = j;
            while j < fm_lines.len() && fm_lines[j].starts_with(' ') {
                j += 1;
            }
            if j > run_start && !owner_opens_block {
                dropped += j - run_start;
                i = j;
                continue;
            }
            kept.push(line);
            i += 1;
            continue;
        }
        if !line.starts_with(' ') {
            let value = line.split_once(':').map_or("", |(_, v)| v.trim());
            owner_opens_block = matches!(value, "" | "|" | "|-" | "|+" | ">" | ">-" | ">+");
        }
        kept.push(line);
        i += 1;
    }
    (dropped > 0).then(|| (format!("---\n{}{tail}", kept.join("\n")), dropped))
}

/// Append `- entry` at the end of `## section`, creating the section when
/// missing (`## log` goes before `## comments`; anything else at EOF).
/// Headings quoted inside fenced code blocks are body content, never the
/// append target.
#[must_use]
pub fn append_section_entry(text: &str, section: &str, entry: &str) -> String {
    let heading = format!("## {section}");
    let lines: Vec<&str> = text.lines().collect();
    let fenced = crate::parse::fenced_lines(&lines);
    let real_heading = |i: &usize, want: &str| !fenced[*i] && lines[*i].trim_end() == want;

    if let Some(h) = (0..lines.len()).find(|i| real_heading(i, heading.as_str())) {
        let section_end = (h + 1..lines.len())
            .find(|i| !fenced[*i] && lines[*i].starts_with("## "))
            .unwrap_or(lines.len());
        let mut insert_at = h + 1;
        for (i, line) in lines.iter().enumerate().take(section_end).skip(h + 1) {
            if !line.trim().is_empty() {
                insert_at = i + 1;
            }
        }
        let mut out: Vec<String> = lines.iter().map(ToString::to_string).collect();
        out.insert(insert_at, format!("- {entry}"));
        return out.join("\n") + "\n";
    }

    // Create the section. `## log` belongs before `## comments` (DESIGN §2).
    if section == "log" {
        if let Some(c) = (0..lines.len()).find(|i| real_heading(i, "## comments")) {
            let mut out: Vec<String> = lines.iter().map(ToString::to_string).collect();
            out.splice(c..c, [heading, format!("- {entry}"), String::new()]);
            return out.join("\n") + "\n";
        }
    }
    let mut out = text.trim_end_matches('\n').to_string();
    let _ = write!(out, "\n\n{heading}\n- {entry}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLANK_BEARING: &str = "---\nid: x\nstatus: open\nhandoff: |\n  one\n\n  two\n\
                                 docs:\n  - A.md\n\n  - B.md\nverify: \"true\"\n---\n\nbody\n";

    #[test]
    fn blocks_with_blank_lines_are_one_value() {
        let out = remove_scalar(BLANK_BEARING, "handoff").unwrap();
        assert_eq!(
            out,
            "---\nid: x\nstatus: open\ndocs:\n  - A.md\n\n  - B.md\nverify: \"true\"\n---\n\nbody\n"
        );
        let out = set_block(BLANK_BEARING, "handoff", &["new".to_string()]).unwrap();
        assert!(out.contains("handoff: |\n  new\ndocs:"), "{out}");
        let out = set_list(BLANK_BEARING, "docs", &["C.md".to_string()]).unwrap();
        assert!(out.contains("  two\ndocs: [C.md]\nverify:"), "{out}");
        let out = append_block_item(BLANK_BEARING, "docs", "C.md").unwrap();
        assert!(out.contains("  - B.md\n  - C.md\nverify:"), "{out}");
    }

    #[test]
    fn trailing_blank_after_block_is_not_consumed() {
        let text = "---\nid: x\nneeds:\n  - a\n\nseq: 10\n---\n";
        assert_eq!(
            remove_scalar(text, "needs").unwrap(),
            "---\nid: x\n\nseq: 10\n---\n"
        );
    }

    #[test]
    fn stranded_block_is_mended_only_under_a_scalar_owner() {
        let damaged = "---\nid: x\nstatus: done\n\n  two\nverify: \"true\"\n---\n";
        let (fixed, n) = strip_stranded_block(damaged).unwrap();
        assert_eq!(n, 1);
        assert_eq!(fixed, "---\nid: x\nstatus: done\nverify: \"true\"\n---\n");
        // The legal shape — blank inside a real block — is left alone.
        assert!(strip_stranded_block(BLANK_BEARING).is_none());
        // A blank between two scalars is legal too.
        assert!(strip_stranded_block("---\nid: x\n\nseq: 1\n---\n").is_none());
    }
}
