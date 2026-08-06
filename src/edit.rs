//! Surgical text edits to task files: one-line frontmatter changes and
//! end-of-section appends. This is MW-I1's discipline made mechanical —
//! a status change must diff as one line so union merges stay clean.

use std::fmt::Write as _;

/// Replace the first `key:` line inside the frontmatter with `key: value`
/// (or a bare `key:` when `value` is None); insert before the closing fence
/// when the key is absent. Everything else is preserved byte-for-byte.
///
/// # Errors
/// When the text has no frontmatter fences — that file is invalid and
/// should be repaired by lint, not edited blind.
pub fn set_scalar(text: &str, key: &str, value: Option<&str>) -> Result<String, String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Err("missing frontmatter fences".to_string());
    };
    let Some(end) = rest.find("\n---") else {
        return Err("missing closing frontmatter fence".to_string());
    };
    let fm = &rest[..end];
    let tail = &rest[end..];

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

/// Remove the first `key:` line from the frontmatter (used when a list
/// empties — a bare `needs: []` is noise in a hand-editable file).
///
/// # Errors
/// When fences are missing, like [`set_scalar`].
pub fn remove_scalar(text: &str, key: &str) -> Result<String, String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Err("missing frontmatter fences".to_string());
    };
    let Some(end) = rest.find("\n---") else {
        return Err("missing closing frontmatter fence".to_string());
    };
    let fm = &rest[..end];
    let tail = &rest[end..];
    let prefix = format!("{key}:");
    let mut removed = false;
    let lines: Vec<&str> = fm
        .lines()
        .filter(|line| {
            if !removed && line.starts_with(&prefix) {
                removed = true;
                false
            } else {
                true
            }
        })
        .collect();
    Ok(format!("---\n{}{tail}", lines.join("\n")))
}

/// Replace `key:` and any indented block under it with an inline list
/// (`key: [a, b]`), inserting before the closing fence when absent. Items
/// pass through [`crate::write::yaml_scalar`].
///
/// # Errors
/// When fences are missing, like [`set_scalar`].
pub fn set_list(text: &str, key: &str, items: &[String]) -> Result<String, String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Err("missing frontmatter fences".to_string());
    };
    let Some(end) = rest.find("\n---") else {
        return Err("missing closing frontmatter fence".to_string());
    };
    let fm = &rest[..end];
    let tail = &rest[end..];

    let rendered = format!(
        "{key}: [{}]",
        items
            .iter()
            .map(|i| crate::write::yaml_scalar(i))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let prefix = format!("{key}:");
    let mut lines: Vec<String> = Vec::new();
    let mut replaced = false;
    let mut skipping_block = false;
    for line in fm.lines() {
        if skipping_block {
            if line.starts_with(' ') {
                continue; // the old block-list items under the key
            }
            skipping_block = false;
        }
        if !replaced && line.starts_with(&prefix) {
            lines.push(rendered.clone());
            replaced = true;
            skipping_block = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(rendered);
    }
    Ok(format!("---\n{}{tail}", lines.join("\n")))
}

/// Replace `key:` and any indented block under it with a literal block
/// scalar (`key: |` + two-space-indented lines), inserting before the
/// closing fence when the key is absent (mw-0f4j: `set --handoff`).
///
/// # Errors
/// When fences are missing, like [`set_scalar`].
pub fn set_block(text: &str, key: &str, block_lines: &[String]) -> Result<String, String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Err("missing frontmatter fences".to_string());
    };
    let Some(end) = rest.find("\n---") else {
        return Err("missing closing frontmatter fence".to_string());
    };
    let fm = &rest[..end];
    let tail = &rest[end..];

    let mut rendered = format!("{key}: |");
    for line in block_lines {
        rendered.push('\n');
        rendered.push_str("  ");
        rendered.push_str(line);
    }
    let prefix = format!("{key}:");
    let mut lines: Vec<String> = Vec::new();
    let mut replaced = false;
    let mut skipping_block = false;
    for line in fm.lines() {
        if skipping_block {
            if line.starts_with(' ') {
                continue;
            }
            skipping_block = false;
        }
        if !replaced && line.starts_with(&prefix) {
            lines.push(rendered.clone());
            replaced = true;
            skipping_block = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        lines.push(rendered);
    }
    Ok(format!("---\n{}{tail}", lines.join("\n")))
}

/// Append one `  - item` to the indented block list under `key:`, creating
/// the key when absent. Existing items — including their trailing `# …`
/// hand comments — are preserved byte-for-byte (mw-0f4j: `set --docs`).
///
/// # Errors
/// When fences are missing, like [`set_scalar`].
pub fn append_block_item(text: &str, key: &str, item: &str) -> Result<String, String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Err("missing frontmatter fences".to_string());
    };
    let Some(end) = rest.find("\n---") else {
        return Err("missing closing frontmatter fence".to_string());
    };
    let fm = &rest[..end];
    let tail = &rest[end..];

    let prefix = format!("{key}:");
    let rendered = format!("  - {item}");
    let fm_lines: Vec<&str> = fm.lines().collect();
    let mut lines: Vec<String> = Vec::new();
    let mut inserted = false;
    let mut i = 0;
    while i < fm_lines.len() {
        let line = fm_lines[i];
        lines.push(line.to_string());
        if !inserted && line.starts_with(&prefix) {
            // copy the existing block, then append after its last item
            let mut j = i + 1;
            while j < fm_lines.len() && fm_lines[j].starts_with(' ') {
                lines.push(fm_lines[j].to_string());
                j += 1;
            }
            lines.push(rendered.clone());
            inserted = true;
            i = j;
            continue;
        }
        i += 1;
    }
    if !inserted {
        lines.push(format!("{key}:"));
        lines.push(rendered);
    }
    Ok(format!("---\n{}{tail}", lines.join("\n")))
}

/// Append `- entry` at the end of `## section`, creating the section when
/// missing (`## log` goes before `## comments`; anything else at EOF).
#[must_use]
pub fn append_section_entry(text: &str, section: &str, entry: &str) -> String {
    let heading = format!("## {section}");
    let lines: Vec<&str> = text.lines().collect();

    if let Some(h) = lines.iter().position(|l| l.trim_end() == heading.as_str()) {
        let section_end = lines[h + 1..]
            .iter()
            .position(|l| l.starts_with("## "))
            .map_or(lines.len(), |off| h + 1 + off);
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
        if let Some(c) = lines.iter().position(|l| l.trim_end() == "## comments") {
            let mut out: Vec<String> = lines.iter().map(ToString::to_string).collect();
            out.splice(c..c, [heading, format!("- {entry}"), String::new()]);
            return out.join("\n") + "\n";
        }
    }
    let mut out = text.trim_end_matches('\n').to_string();
    let _ = write!(out, "\n\n{heading}\n- {entry}\n");
    out
}
