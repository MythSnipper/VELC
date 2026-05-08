use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Debug, Clone)]
pub struct Preprocessor {}

#[derive(Default, Debug)]
struct PreprocessorContext {
    // Files currently being processed. Used for real circular include detection.
    active_paths: HashSet<PathBuf>,

    // Files that had #once and were already processed.
    once_paths: HashSet<PathBuf>,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, fpath: &str, ipaths: &Vec<String>) -> Result<String, String> {
        self.process(fpath, ipaths)
    }

    fn process(&self, filepath: &str, include_paths: &Vec<String>) -> Result<String, String> {
        let mut ctx = PreprocessorContext::default();
        self.process_file(Path::new(filepath), include_paths, &mut ctx)
    }

    fn process_file(
        &self,
        filepath: &Path,
        include_paths: &Vec<String>,
        ctx: &mut PreprocessorContext,
    ) -> Result<String, String> {
        let filepath = fs::canonicalize(filepath).map_err(|_| {
            format!(
                "velc: Preprocessor: Failed to open file: {}",
                filepath.display()
            )
        })?;

        if ctx.once_paths.contains(&filepath) {
            return Ok(String::new());
        }

        if ctx.active_paths.contains(&filepath) {
            return Err(format!(
                "velc: Preprocessor: Circular include detected: {} is already in active include stack",
                filepath.display()
            ));
        }

        ctx.active_paths.insert(filepath.clone());

        let source = fs::read_to_string(&filepath).map_err(|_| {
            format!(
                "velc: Preprocessor: Failed to read file: {}",
                filepath.display()
            )
        })?;

        let mut output = String::new();

        // Defines are local to this file.
        let mut defines: HashMap<String, String> = HashMap::new();

        let mut file_has_once = false;

        for line in source.lines() {
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                if trimmed == "#once" {
                    file_has_once = true;
                    output.push('\n');
                    continue;
                }

                if trimmed.starts_with("#include:") {
                    let keyword_len = "#include:".len();
                    let new_filepath = trimmed[keyword_len..].trim();

                    let new_filepath = self.find_include_file(new_filepath, include_paths)?;

                    let included_source = self.process_file(&new_filepath, include_paths, ctx)?;

                    output.push_str(&included_source);

                    if !included_source.ends_with('\n') {
                        output.push('\n');
                    }

                    continue;
                }

                if trimmed.starts_with("#define") {
                    self.handle_define(trimmed, &mut defines, &filepath)?;
                    output.push('\n');
                    continue;
                }

                // Keep old behavior: unknown preprocessor directives are ignored.
                output.push('\n');
                continue;
            }

            let expanded = self.expand_defines_in_line(line, &defines);

            output.push_str(&expanded);
            output.push('\n');
        }

        ctx.active_paths.remove(&filepath);

        if file_has_once {
            ctx.once_paths.insert(filepath);
        }

        Ok(output)
    }

    fn handle_define(
        &self,
        line: &str,
        defines: &mut HashMap<String, String>,
        filepath: &Path,
    ) -> Result<(), String> {
        let rest = line
            .strip_prefix("#define")
            .unwrap()
            .trim();

        if rest.is_empty() {
            return Err(format!(
                "velc: Preprocessor: Invalid #define in {}",
                filepath.display()
            ));
        }

        let mut parts = rest.splitn(2, char::is_whitespace);

        let name = parts.next().unwrap().trim();
        let value = parts.next().unwrap_or("").trim();

        if !Self::is_identifier(name) {
            return Err(format!(
                "velc: Preprocessor: Invalid macro name `{}` in {}",
                name,
                filepath.display()
            ));
        }

        defines.insert(name.to_string(), value.to_string());

        Ok(())
    }

    fn expand_defines_in_line(
        &self,
        line: &str,
        defines: &HashMap<String, String>,
    ) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut output = String::new();

        let mut i = 0;

        let mut in_string = false;
        let mut in_char = false;
        let mut escaped = false;

        while i < chars.len() {
            let ch = chars[i];

            if in_string {
                output.push(ch);

                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }

                i += 1;
                continue;
            }

            if in_char {
                output.push(ch);

                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '\'' {
                    in_char = false;
                }

                i += 1;
                continue;
            }

            if ch == '"' {
                in_string = true;
                output.push(ch);
                i += 1;
                continue;
            }

            if ch == '\'' {
                in_char = true;
                output.push(ch);
                i += 1;
                continue;
            }

            if Self::is_identifier_start(ch) {
                let start = i;
                i += 1;

                while i < chars.len() && Self::is_identifier_continue(chars[i]) {
                    i += 1;
                }

                let ident: String = chars[start..i].iter().collect();

                if let Some(replacement) = defines.get(&ident) {
                    output.push_str(replacement);
                } else {
                    output.push_str(&ident);
                }

                continue;
            }

            output.push(ch);
            i += 1;
        }

        output
    }

    fn find_include_file(
        &self,
        filepath: &str,
        include_paths: &Vec<String>,
    ) -> Result<PathBuf, String> {
        let direct_path = Path::new(filepath);

        if direct_path.exists() {
            return fs::canonicalize(direct_path).map_err(|_| {
                format!(
                    "velc: Preprocessor: Failed to canonicalize include file {}",
                    filepath
                )
            });
        }

        for include_path in include_paths {
            let candidate = Path::new(include_path).join(filepath);

            if candidate.exists() {
                return fs::canonicalize(&candidate).map_err(|_| {
                    format!(
                        "velc: Preprocessor: Failed to canonicalize include file {}",
                        candidate.display()
                    )
                });
            }
        }

        Err(format!(
            "velc: Preprocessor: Couldn't find include file {} in find_include_file",
            filepath
        ))
    }

    fn is_identifier(s: &str) -> bool {
        let mut chars = s.chars();

        let Some(first) = chars.next() else {
            return false;
        };

        if !Self::is_identifier_start(first) {
            return false;
        }

        for ch in chars {
            if !Self::is_identifier_continue(ch) {
                return false;
            }
        }

        true
    }

    fn is_identifier_start(ch: char) -> bool {
        ch == '_' || ch.is_ascii_alphabetic()
    }

    fn is_identifier_continue(ch: char) -> bool {
        ch == '_' || ch.is_ascii_alphanumeric()
    }
}