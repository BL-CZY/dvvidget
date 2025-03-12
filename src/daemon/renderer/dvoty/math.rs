use core::f64;
use std::sync::Arc;

use evalexpr::{context_map, Value};
use gtk4::{prelude::DisplayExt, ListBox};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::daemon::{
    renderer::{config::AppConf, dvoty::entry::DvotyEntry},
    structs::{DaemonCmdType, DaemonEvt, Dvoty},
};

use super::entry::DvotyUIEntry;
use super::{class::adjust_class, DvotyContext};

pub fn set_clipboard_text(text: &str) {
    let display = gtk4::gdk::Display::default().expect("Could not get default display");
    let clipboard = display.clipboard();

    clipboard.set_text(text);
}

pub fn preprocess_math(input: &str) -> String {
    input
        .replace(" ", "")
        // Math functions
        .replace("is_nan", "math::is_nan")
        .replace("is_finite", "math::is_finite")
        .replace("is_infinite", "math::is_infinite")
        .replace("is_normal", "math::is_normal")
        .replace("ln", "math::ln")
        .replace("log", "math::log")
        .replace("log2", "math::log2")
        .replace("log10", "math::log10")
        .replace("exp", "math::exp")
        .replace("exp2", "math::exp2")
        .replace("pow", "math::pow")
        .replace("cos", "math::cos")
        .replace("acos", "math::acos")
        .replace("cosh", "math::cosh")
        .replace("acosh", "math::acosh")
        .replace("sin", "math::sin")
        .replace("asin", "math::asin")
        .replace("sinh", "math::sinh")
        .replace("asinh", "math::asinh")
        .replace("tan", "math::tan")
        .replace("atan", "math::atan")
        .replace("atan2", "math::atan2")
        .replace("tanh", "math::tanh")
        .replace("atanh", "math::atanh")
        .replace("sqrt", "math::sqrt")
        .replace("cbrt", "math::cbrt")
        .replace("hypot", "math::hypot")
        .replace("abs", "math::abs")
        // String functions
        .replace("regex_matches", "str::regex_matches")
        .replace("regex_replace", "str::regex_replace")
        .replace("to_lowercase", "str::to_lowercase")
        .replace("to_uppercase", "str::to_uppercase")
        .replace("trim", "str::trim")
        .replace("from", "str::from")
        .replace("substring", "str::substring")
}

pub fn post_process_result(input: Value) -> String {
    match input {
        Value::Float(val) => format!("{:.8}", val).trim_end_matches('0').to_string(),
        _ => input.to_string(),
    }
}

pub fn eval_math(input: String, sender: UnboundedSender<DaemonEvt>, id: &Uuid, monitor: usize) {
    use evalexpr::Value;
    let input = preprocess_math(&input);
    let context = match context_map! {
        "e" => Value::Float(f64::consts::E),
        "pi" => Value::Float(f64::consts::PI),
        "deg" => Function::new(|argument| {
            let arguments = argument.as_number()?;

            Ok(Value::Float(arguments / 180f64 * f64::consts::PI))
        }),
        "avg" => Function::new(|argument| {
            let arguments = argument.as_tuple()?;

            if arguments.is_empty() {
                return Err(evalexpr::EvalexprError::CustomMessage("Average of empty set is undefined".to_string()));
            }

            let sum: f64 = arguments.iter()
                .map(|arg| arg.as_number())
                .collect::<Result<Vec<f64>, evalexpr::EvalexprError>>()?
                .iter()
                .sum();

            let avg = sum / arguments.len() as f64;

            Ok(Value::Float(avg))
        }),
    } {
        Ok(res) => res,
        Err(e) => {
            println!("Dvoty: Error creating math context: {}", e);
            return;
        }
    };

    match evalexpr::eval_with_context(&input, &context) {
        Ok(res) => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Math {
                        expression: input,
                        result: post_process_result(res),
                    })),
                    sender: None,
                    uuid: Some(*id),
                    monitors: vec![monitor],
                })
                .unwrap_or_else(|e| println!("Dvoty: Failed to send math result: {}", e));
        }
        Err(e) => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Math {
                        expression: input,
                        result: e.to_string(),
                    })),
                    sender: None,
                    uuid: Some(*id),
                    monitors: vec![monitor],
                })
                .unwrap_or_else(|e| println!("Dvoty: Failed to send math result: {}", e));
        }
    }
}

pub fn populate_math_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    result: String,
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) {
    let row = super::entry::create_base_entry(
        &config.dvoty.math_icon,
        &format!("={}", &result),
        "Click to copy",
        sender,
        config.clone(),
        monitor,
    );

    let result_clone = result.clone();

    context.dvoty_entries[monitor].push((
        DvotyUIEntry::Math {
            result: result_clone,
        },
        row.clone(),
    ));

    if context.dvoty_entries.len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty_entries[monitor]);
    }

    list.append(&row);
}
