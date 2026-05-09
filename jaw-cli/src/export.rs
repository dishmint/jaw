use crate::lang::{BlockComment, LangSpec};

pub fn export(source: &str, lang: &LangSpec) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let n = lines.len();
    let mut out = String::with_capacity(source.len() * 2);
    let mut i = 0;

    while i < n {
        let line = lines[i];
        let trimmed = line.trim_start();

        if (trimmed.starts_with("[*]") || trimmed.starts_with("[^]")) && lang.block.is_some() {
            let end = continuation_end(&lines, i + 1);
            if end > i + 1 {
                emit_block(&mut out, &lines[i..end], lang.block.as_ref().unwrap());
                i = end;
                continue;
            }
        }

        emit_line(&mut out, line, lang);
        i += 1;
    }

    if source.ends_with('\n') && !out.ends_with('\n') {
        out.push('\n');
    }

    out
}

fn continuation_end(lines: &[&str], from: usize) -> usize {
    let mut end = from;
    while end < lines.len() {
        let l = lines[end];
        if l.trim().is_empty() {
            break;
        }
        if is_marker_line(l) {
            break;
        }
        end += 1;
    }
    end
}

fn is_marker_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('[')
        || t.starts_with('/')
        || t.starts_with("```")
        || t.starts_with("~~~")
}

fn split_indent(line: &str) -> (&str, &str) {
    let i = line.len() - line.trim_start().len();
    line.split_at(i)
}

fn emit_line(out: &mut String, line: &str, lang: &LangSpec) {
    if line.trim().is_empty() {
        out.push('\n');
        return;
    }
    let (indent, content) = split_indent(line);
    out.push_str(indent);
    out.push_str(lang.line_prefix);
    out.push(' ');
    out.push_str(content);
    out.push('\n');
}

fn emit_block(out: &mut String, block_lines: &[&str], block: &BlockComment) {
    let (indent, first_content) = split_indent(block_lines[0]);
    out.push_str(indent);
    out.push_str(block.open);
    out.push(' ');
    out.push_str(first_content);
    out.push('\n');

    let last_idx = block_lines.len() - 1;
    for line in &block_lines[1..last_idx] {
        out.push_str(line);
        out.push('\n');
    }

    out.push_str(block_lines[last_idx]);
    out.push(' ');
    out.push_str(block.close);
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::lookup;

    #[test]
    fn python_per_line() {
        let src = "[V] — a vector\n[!] — done\n";
        let out = export(src, lookup("python").unwrap());
        assert_eq!(out, "# [V] — a vector\n# [!] — done\n");
    }

    #[test]
    fn rust_per_line() {
        let src = "[V] — a vector\n";
        let out = export(src, lookup("rust").unwrap());
        assert_eq!(out, "// [V] — a vector\n");
    }

    #[test]
    fn preserves_indentation() {
        let src = "/foo\n\t[1] — step\n";
        let out = export(src, lookup("python").unwrap());
        assert_eq!(out, "# /foo\n\t# [1] — step\n");
    }

    #[test]
    fn preserves_blank_lines() {
        let src = "[V] — a\n\n[T] — b\n";
        let out = export(src, lookup("python").unwrap());
        assert_eq!(out, "# [V] — a\n\n# [T] — b\n");
    }

    #[test]
    fn multiline_general_comment_becomes_block_in_c_style() {
        let src = "[*] — first line\ncontinuation line\nlast line\n[1] — step\n";
        let out = export(src, lookup("rust").unwrap());
        assert_eq!(
            out,
            "/* [*] — first line\ncontinuation line\nlast line */\n// [1] — step\n"
        );
    }

    #[test]
    fn multiline_general_comment_stays_per_line_in_hash_lang() {
        let src = "[*] — first line\ncontinuation line\n[1] — step\n";
        let out = export(src, lookup("python").unwrap());
        assert_eq!(
            out,
            "# [*] — first line\n# continuation line\n# [1] — step\n"
        );
    }

    #[test]
    fn single_line_general_comment_stays_per_line() {
        let src = "[*] — just one line\n[1] — step\n";
        let out = export(src, lookup("rust").unwrap());
        assert_eq!(out, "// [*] — just one line\n// [1] — step\n");
    }

    #[test]
    fn blank_line_terminates_multiline_block() {
        let src = "[*] — opens\ncontinues\n\n[1] — step\n";
        let out = export(src, lookup("rust").unwrap());
        assert_eq!(out, "/* [*] — opens\ncontinues */\n\n// [1] — step\n");
    }

    #[test]
    fn code_comment_marker_also_blocks() {
        let src = "[^] explanation\nspans two lines\n[1] — step\n";
        let out = export(src, lookup("rust").unwrap());
        assert_eq!(
            out,
            "/* [^] explanation\nspans two lines */\n// [1] — step\n"
        );
    }

    #[test]
    fn unknown_lang_returns_none() {
        assert!(lookup("klingon").is_none());
    }

    #[test]
    fn lang_aliases_work() {
        assert!(lookup("py").is_some());
        assert!(lookup("rb").is_some());
        assert!(lookup("ts").is_some());
        assert!(lookup("PYTHON").is_some());
    }
}
