use colored::Colorize;
use glob::glob;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use walkdir::WalkDir;

struct Config {
    pattern: String,
    files: Vec<PathBuf>,
    case_insensitive: bool,      // -i: Case-insensitive search
    show_line_numbers: bool,     // -n: Show line numbers
    invert_match: bool,          // -v: Invert match (exclude matching lines)
    recursive: bool,             // -r: Recursive directory search
    show_filenames: bool,        // -f: Show filenames
    colored_output: bool,        // -c: Enable colored output
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        let mut pattern = String::new();
        let mut files = Vec::new();
        let mut case_insensitive = false;
        let mut show_line_numbers = false;
        let mut invert_match = false;
        let mut recursive = false;
        let mut show_filenames = false;
        let mut colored_output = false;

        let mut i = 1;
        while i < args.len() {
            if args[i].starts_with('-') {
                match args[i].as_str() {
                    "-i" => case_insensitive = true,
                    "-n" => show_line_numbers = true,
                    "-v" => invert_match = true,
                    "-r" => recursive = true,
                    "-f" => show_filenames = true,
                    "-c" => colored_output = true,
                    "-h" | "--help" => println!("{}", Self::help_message()),
                    _ => return Err("Unknown option"),
                }
            } else if pattern.is_empty() {
                pattern = args[i].clone();
            } else {
                for entry in glob(&args[i]).expect("Failed to read glob pattern") {
                    match entry {
                        Ok(path) => files.push(path),
                        Err(_) => continue,
                    }
                }
            }
            i += 1;
        }

        Ok(Config {
            pattern,
            files,
            case_insensitive,
            show_line_numbers,
            invert_match,
            recursive,
            show_filenames,
            colored_output,
        })
    }

    fn help_message() -> &'static str {
        r#"Usage: grep [OPTIONS] <pattern> <files...>

Options:
-i                Case-insensitive search
-n                Print line numbers
-v                Invert match (exclude lines that match the pattern)
-r                Recursive directory search
-f                Print filenames
-c                Enable colored output
-h, --help        Show help information"#
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::new(&args) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    if config.recursive {
        recursive_search(&config);
    } else {
        for file_path in &config.files {
            search_file(file_path, &config);
        }
    }
}

fn search_file(path: &PathBuf, config: &Config) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return,
    };
    let reader = BufReader::new(file);

    for (num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(line) => line,
            Err(_) => continue,
        };

        let matches = if config.case_insensitive {
            line.to_lowercase().contains(&config.pattern.to_lowercase())
        } else {
            line.contains(&config.pattern)
        };

        if matches != config.invert_match {
            let output = format_output(&line, num + 1, &path.to_string_lossy(), config);
            println!("{}", output);
        }
    }
}

fn recursive_search(config: &Config) {
    for file_path in &config.files {
        for entry in WalkDir::new(file_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "md") {
                search_file(&entry.path().to_path_buf(), config);
            }
        }
    }
}

fn format_output(line: &str, num: usize, file_name: &str, config: &Config) -> String {
    let mut output = String::new();

    if config.show_filenames {
        output.push_str(&format!("{}: ", file_name));
    }

    if config.show_line_numbers {
        output.push_str(&format!("{}: ", num));
    }

    if config.colored_output {
        let colored_line = if config.case_insensitive {
            let re = regex::Regex::new(&regex::escape(&config.pattern.to_lowercase())).unwrap();
            let line_lower = line.to_lowercase();
            let mut result = String::new();
            let mut last = 0;
            for mat in re.find_iter(&line_lower) {
                result.push_str(&line[last..mat.start()]);
                result.push_str(&line[mat.start()..mat.end()].red().to_string());
                last = mat.end();
            }
            result.push_str(&line[last..]);
            result
        } else {
            line.replace(&config.pattern, &config.pattern.red().to_string())
        };
        output.push_str(&colored_line);
    } else {
        output.push_str(line);
    }

    output
}