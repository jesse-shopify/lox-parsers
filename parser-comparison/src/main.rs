//! Parser comparison tool for Lox language implementations

use clap::{Arg, Command};
use colored::*;
use parser_comparison::{compare_parsers, get_all_parsers, ComparisonResult};
use std::fs;
use std::io::{self, Read};

fn main() {
    let matches = Command::new("parser-comparison")
        .about("Compare Lox parser implementations for accuracy and performance")
        .version("0.1.0")
        .arg(
            Arg::new("input")
                .help("Input file to parse, or '-' for interactive mode")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output results in JSON format")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();


    let input = match matches.get_one::<String>("input") {
        Some(path) if path == "-" => {
            println!("{}", "Interactive mode - enter Lox code (Ctrl+D to finish):".cyan());
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).unwrap();
            input
        }
        Some(path) => match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("{}: {}", "Error reading file".red(), e);
                std::process::exit(1);
            }
        },
        None => {
            println!("{}", "No input provided. Use --help for usage information.".yellow());
            println!("{}", "Run 'cargo test' to execute the parser comparison test suite.".cyan());
            std::process::exit(1);
        }
    };

    let parsers = get_all_parsers();
    let result = compare_parsers(&input, parsers);

    if matches.get_flag("json") {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        print_colored_result(&result);
    }
}


fn print_colored_result(result: &ComparisonResult) {
    println!("{}", "=== Parser Comparison Results ===".cyan().bold());
    println!("{} {}", "Input:".blue(), result.input.trim().white());
    println!();

    println!("{}", "Summary:".green().bold());
    println!("  {}: {}", "Total parsers".blue(), result.accuracy_summary.total_parsers);
    println!("  {}: {}", "Successful".green(), result.accuracy_summary.successful_parsers);
    println!("  {}: {}", "Failed".red(), result.accuracy_summary.failed_parsers);
    println!("  {}: {}", "Consensus reached".yellow(),
        if result.accuracy_summary.consensus_reached { "✅ Yes".green() } else { "❌ No".red() }
    );
    println!("  {}: {:?}", "Average parse time".blue(), result.accuracy_summary.average_parse_time);
    println!();

    println!("{}", "Individual Results:".green().bold());
    for parser_result in &result.results {
        let status = if parser_result.result.success {
            "✅ SUCCESS".green()
        } else {
            "❌ FAILED".red()
        };

        println!("  {} v{}: {} ({:?})",
            parser_result.parser_name.cyan(),
            parser_result.parser_version.white(),
            status,
            parser_result.result.parse_time
        );

        if let Some(ref error) = parser_result.result.error_message {
            println!("    {}: {}", "Error".red(), error.white());
        }
    }

    if let Some(ref ast) = result.consensus_ast {
        println!();
        println!("{}", "Consensus AST:".green().bold());
        println!("{:#?}", ast);
    } else if result.accuracy_summary.successful_parsers > 1 {
        println!();
        println!("{}", "⚠️  No consensus: Parsers produced different ASTs".yellow());
    }
}

