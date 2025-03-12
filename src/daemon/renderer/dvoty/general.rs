use std::{collections::HashSet, path::PathBuf, sync::Arc};

use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::{
    renderer::config::AppConf,
    structs::{DaemonCmdType, DaemonEvt, Dvoty},
};

use super::{
    app_launcher::process_apps, files::process_recent_files, letter::process_greek_letters,
    math::eval_math, search::process_history, DvotyEntry,
};

lazy_static::lazy_static! {
    pub static ref MATH_FUNCTIONS: HashSet<&'static str> = {
        // Common math functions
        let math_functions = vec![
            "avg",
            // Basic functions
            "min",
            "max",
            "len",
            "floor",
            "round",
            "ceil",
            "if",
            "contains",
            "contains_any",
            "typeof",
            "random",
            // Math functions (without the math:: prefix)
            "is_nan",
            "is_finite",
            "is_infinite",
            "is_normal",
            "ln",
            "log",
            "log2",
            "log10",
            "exp",
            "exp2",
            "pow",
            "cos",
            "acos",
            "cosh",
            "acosh",
            "sin",
            "asin",
            "sinh",
            "asinh",
            "tan",
            "atan",
            "atan2",
            "tanh",
            "atanh",
            "sqrt",
            "cbrt",
            "hypot",
            "abs",
            // String functions
            "regex_matches",
            "regex_replace",
            "to_lowercase",
            "to_uppercase",
            "trim",
            "from",
            "substring",
            // Bitwise operations
            "bitand",
            "bitor",
            "bitxor",
            "bitnot",
            "shl",
            "shr",
            // Legacy functions (still include them without prefix)
            "sin",
            "cos",
            "tan",
            "log",
            "ln",
            "exp",
            "sqrt",
        ];
        math_functions.into_iter().collect()
    };
}

fn is_mathable(input: &str) -> bool {
    // If the string is empty, it's not a math expression
    if input.trim().is_empty() {
        return false;
    }

    // Common math operators and symbols
    let math_operators = ['+', '-', '*', '/', '=', '<', '>', '^', '√', '(', ')'];

    // Count math-related characters and symbols
    let mut math_char_count = 0;
    let mut has_digit = false;
    let mut potential_function = String::new();

    for c in input.chars() {
        if c.is_ascii_digit() {
            has_digit = true;
            math_char_count += 1;
        } else if math_operators.contains(&c) {
            math_char_count += 1;
        } else if c.is_alphabetic() {
            potential_function.push(c);
        } else if c == '(' || c == ')' {
            math_char_count += 1;
        } else if c == '.' && potential_function.is_empty() {
            // Might be part of a decimal number
            math_char_count += 1;
        } else if c.is_whitespace() {
            // Check if the collected letters form a math function
            if !potential_function.is_empty() {
                if MATH_FUNCTIONS.contains(potential_function.as_str()) {
                    math_char_count += potential_function.len();
                }
                potential_function.clear();
            }
        }
    }

    // Check if the last collected letters form a math function
    if !potential_function.is_empty() && MATH_FUNCTIONS.contains(potential_function.as_str()) {
        math_char_count += potential_function.len();
    }

    // Heuristic: if at least 30% of the non-whitespace characters are math-related
    // and there's at least one digit or a known math function, consider it a math expression
    let non_whitespace_count = input.chars().filter(|c| !c.is_whitespace()).count();

    if non_whitespace_count == 0 {
        return false;
    }

    let math_ratio = math_char_count as f64 / non_whitespace_count as f64;

    // Consider it a math expression if:
    // 1. At least 30% of characters are math-related AND
    // 2. It contains at least one digit or a recognized math function
    math_ratio >= 0.3 && (has_digit || MATH_FUNCTIONS.iter().any(|&f| input.contains(f)))
}

pub async fn process_general(
    sender: UnboundedSender<DaemonEvt>,
    input: &str,
    id: &uuid::Uuid,
    config: Arc<AppConf>,
    monitor: usize,
    recent_paths: Vec<PathBuf>,
) {
    // math
    if is_mathable(input) && config.dvoty.general_options.math {
        eval_math(input.to_lowercase(), sender.clone(), id, monitor);
    }

    // letter
    if config.dvoty.general_options.letter {
        process_greek_letters(input.to_string(), sender.clone(), id, monitor);
    }

    // app launcher
    if config.dvoty.general_options.launch {
        process_apps(input, sender.clone(), id, config.clone(), monitor);
    }

    // search
    if config.dvoty.general_options.search {
        sender
            .send(DaemonEvt {
                evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Search {
                    keyword: input.into(),
                })),
                sender: None,
                uuid: Some(*id),
                monitors: vec![monitor],
            })
            .unwrap_or_else(|e| {
                println!("Dvoty: Error adding search entry: {}", e);
            });
    }

    // recent files
    if config.dvoty.general_options.files {
        process_recent_files(input.to_string(), sender.clone(), id, monitor, recent_paths);
    }

    // website
    process_history(
        input,
        config.clone(),
        sender,
        id,
        monitor,
        config.dvoty.general_options.history,
        config.dvoty.general_options.bookmark,
    )
    .await
    .unwrap_or_else(|e| {
        println!("{}", e);
    });
}
