use mur_lang::lexer::{tokenize, Token as LexerToken};
use mur_lang::parser::parse;
use mur_lang::interpreter::MurlocRuntime;
use mur_lang::value_parser::ParseError;
use std::time::Instant;
use std::env;
use std::fs;
use env_logger::Env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

enum LogLevel {
    Info,
    Error
}

fn log(level: LogLevel, message: &str) {
    let prefix = match level {
        LogLevel::Info => "[INFO]",
        LogLevel::Error => "[ERROR]"
    };
    println!("{} {}", prefix, message);
}

fn show_help() {
    println!("Murlang v{} (Beta) - A programming language for the murloc tribe", VERSION);
    println!("\nUsage:");
    println!("  mrgl run <file.mur>    Run a Murlang program");
    println!("  mrgl help              Show this help message");
    println!("  mrgl --version         Show version information");
    println!("\nExamples:");
    println!("  mrgl run hello.mur     Run the hello.mur program");
    println!("  mrgl help              Show this help message");
    println!("\nMrglglglgl! For more information, visit: https://github.com/GabrielEstefanski/murlang");
}

fn show_version() {
    println!("Murlang v{} (Beta)", VERSION);
    println!("Mrglglglgl! A programming language for the murloc tribe");
    println!("\nThis is a beta version. Please report any issues at:");
    println!("https://github.com/GabrielEstefanski/murlang/issues");
}

fn main() -> Result<(), ParseError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "help" => {
                show_help();
                return Ok(());
            }
            "--version" | "-V" => {
                show_version();
                return Ok(());
            }
            _ => {}
        }
    }

    let file_path = args.iter().find(|s| s.ends_with(".mur"));
    let source = match file_path {
        Some(path) => match fs::read_to_string(path) {
            Ok(content) => {
                log(LogLevel::Info, &format!("Executing file: {}", path));
                content
            },
            Err(err) => {
                log(LogLevel::Error, &format!("Failed to read file: {}", err));
                return Err(ParseError::InvalidValue(format!("File read error: {}", err)));
            }
        },
        None => {
            log(LogLevel::Info, "No file provided.");
            show_help();
            return Ok(());
        }
    };

    let total_start = Instant::now();

    let start = Instant::now();
    log(LogLevel::Info, "Tokenizing code...");
    let spanned_tokens = tokenize(&source)
        .map_err(|e| ParseError::InvalidValue(format!(
            "Lexer error at line {}, column {}: {}", e.line, e.column, e.message)))?;
    log(LogLevel::Info, &format!("Tokenizing completed in {:.2?}", start.elapsed()));

    let start = Instant::now();
    log(LogLevel::Info, "Parsing code...");
    let tokens: Vec<LexerToken> = spanned_tokens.iter().map(|t| t.token.clone()).collect();
    let statements = parse(tokens)?;
    log(LogLevel::Info, &format!("Parsing completed in {:.2?}", start.elapsed()));

    let start = Instant::now();
    log(LogLevel::Info, "Executing code...");
    let runtime = MurlocRuntime::new();
    runtime.run(statements)?;
    log(LogLevel::Info, &format!("Execution completed in {:.2?}", start.elapsed()));

    log(LogLevel::Info, &format!("Total runtime: {:.2?}", total_start.elapsed()));
    Ok(())
}