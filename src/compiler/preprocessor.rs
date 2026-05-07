use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};



#[derive(Default, Debug, Clone)]
pub struct Preprocessor {
    
}
impl Preprocessor {
    pub fn new() -> Self {
        Self {
            
        }
    }

    pub fn run(&self, fpath: &str, ipaths: &Vec<String>) -> Result<String, String> {
        self.process(fpath, ipaths)
    }

    fn process(&self, filepath: &str, include_paths: &Vec<String>) -> Result<String, String> {
        let mut visited_paths = HashSet::new();
        self.process_file(Path::new(filepath), include_paths, &mut visited_paths)
    }

    fn process_file(&self, filepath: &Path, include_paths: &Vec<String>, visited_paths: &mut HashSet<PathBuf>) -> Result<String, String> {
        let filepath = fs::canonicalize(filepath).map_err(|_| {
            format!(
                "velc: Preprocessor: Failed to open file: {}",
                filepath.display()
            )
        })?;

        if visited_paths.contains(&filepath) {
            return Err(format!(
                "velc: Preprocessor: Circular include detected: {} already visited",
                filepath.display()
            ));
        }

        visited_paths.insert(filepath.clone());

        let source = fs::read_to_string(&filepath).map_err(|_| {
            format!(
                "velc: Preprocessor: Failed to read file: {}",
                filepath.display()
            )
        })?;

        let mut output = String::new();

        for line in source.lines() {
            if line.starts_with('#') {
                if line.starts_with("#include:") {
                    let keyword_len = "#include:".len();

                    let new_filepath = line[keyword_len..].trim();

                    let new_filepath = self.find_include_file(new_filepath, include_paths)?;

                    let included_source = self.process_file(&new_filepath, include_paths, visited_paths)?;

                    output.push_str(&included_source);

                    if !included_source.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
            else {
                output.push_str(line);
                output.push('\n');
            }
        }

        Ok(output)
    }

    fn find_include_file(&self, filepath: &str, include_paths: &Vec<String>) -> Result<PathBuf, String> {
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


}

