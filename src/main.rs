use mur_lang::lexer::{tokenize, Token as LexerToken};
use mur_lang::parser::parse;
use mur_lang::interpreter::MurlocRuntime;
use mur_lang::value_parser::ParseError;
use std::time::Instant;
use std::env;
use std::fs;
use env_logger::Env;

enum LogLevel {
    Info,
    Error,
    Debug,
}

fn log(level: LogLevel, message: &str) {
    let prefix = match level {
        LogLevel::Info => "[INFO]",
        LogLevel::Error => "[ERROR]",
        LogLevel::Debug => "[DEBUG]",
    };
    println!("{} {}", prefix, message);
}

fn main() -> Result<(), ParseError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    let file_path = args.iter().find(|s| s.ends_with(".mur"));
    let source = match file_path {
        Some(path) => match fs::read_to_string(path) {
            Ok(content) => {
                log(LogLevel::Info, &format!("Mrglglglgl! Executing file: {}", path));
                content
            },
            Err(err) => {
                log(LogLevel::Error, &format!("Failed to read file: {}", err));
                return Err(ParseError::InvalidValue(format!("File read error: {}", err)));
            }
        },
        None => {
            log(LogLevel::Info, "Mrglglglgl! No file provided, nothing to execute.");
            return Ok(());
        }
    };

    let total_start = Instant::now();

    let start = Instant::now();
    log(LogLevel::Info, "Mrglglgl! Tokenizing code...");
    let spanned_tokens = tokenize(&source)
        .map_err(|e| ParseError::InvalidValue(format!(
            "Lexer error at line {}, column {}: {}", e.line, e.column, e.message)))?;
    log(LogLevel::Info, &format!("Tokenizing completed in {:.2?}", start.elapsed()));

    if verbose {
        log(LogLevel::Debug, "First tokens:");
        for (i, token) in spanned_tokens.iter().take(10).enumerate() {
            log(LogLevel::Debug, &format!(
                "{}: {:?} (line {}, column {})",
                i, token.token, token.line, token.column
            ));
        }
    }

    let start = Instant::now();
    log(LogLevel::Info, "Aaaaaughibbrgubugbugrguburgle! Parsing code...");
    let tokens: Vec<LexerToken> = spanned_tokens.iter().map(|t| t.token.clone()).collect();
    let statements = parse(tokens)?;
    log(LogLevel::Info, &format!("Parsing completed in {:.2?}", start.elapsed()));

    let start = Instant::now();
    log(LogLevel::Info, "Mrglglgl! Executing code...");
    let runtime = MurlocRuntime::new();
    runtime.run(statements)?;
    log(LogLevel::Info, &format!("Execution completed in {:.2?}", start.elapsed()));

    log(LogLevel::Info, &format!("Total runtime: {:.2?}", total_start.elapsed()));
    Ok(())
}