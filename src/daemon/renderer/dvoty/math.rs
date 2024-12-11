use core::f64;
use std::{cell::RefMut, sync::Arc};

use evalexpr::{context_map, Value};
use gtk4::{prelude::DisplayExt, ListBox};
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::{
    renderer::{app::AppContext, config::AppConf, dvoty::entry::DvotyEntry},
    structs::{DaemonCmd, DaemonEvt, Dvoty},
};

use super::class::adjust_class;
use super::entry::DvotyUIEntry;

pub fn set_clipboard_text(text: &str) {
    let display = gtk4::gdk::Display::default().expect("Could not get default display");
    let clipboard = display.clipboard();

    clipboard.set_text(text);
}

fn preprocess_math(input: &str) -> String {
    input
        .replace(" ", "")
        .replace("ln", "math::ln")
        .replace("log", "math::log")
        .replace("sin", "math::sin")
        .replace("cos", "math::cos")
        .replace("tan", "math::tan")
        .replace("sqrt", "math::sqrt")
}

fn post_process_result(input: Value) -> String {
    match input {
        Value::Float(val) => format!("{:.8}", val).trim_end_matches('0').to_string(),
        _ => input.to_string(),
    }
}

pub fn eval_math(input: &str, sender: UnboundedSender<DaemonEvt>) {
    use evalexpr::Value;
    let input = preprocess_math(input);
    let context = match context_map! {
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

    let expr = input.chars().skip(1).collect::<String>();
    match evalexpr::eval_with_context(&expr, &context) {
        Ok(res) => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Math {
                        expression: expr,
                        result: post_process_result(res),
                    })),
                    sender: None,
                })
                .unwrap_or_else(|e| println!("Dvoty: Failed to send math result: {}", e));
        }
        Err(e) => {
            println!("Dvoty: Failed to evaluate math: {}", e);
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Math {
                        expression: expr,
                        result: e.to_string(),
                    })),
                    sender: None,
                })
                .unwrap_or_else(|e| println!("Dvoty: Failed to send math result: {}", e));
        }
    }
}

pub fn populate_math_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    result: String,
    context: &mut RefMut<AppContext>,
    sender: UnboundedSender<DaemonEvt>,
) {
    let row =
        super::entry::create_base_entry(&config.dvoty.math_icon, &result, "Click to copy", sender);

    let result_clone = result.clone();

    context.dvoty.dvoty_entries.push((
        DvotyUIEntry::Math {
            result: result_clone,
        },
        row.clone(),
    ));

    adjust_class(0, 0, &mut context.dvoty.dvoty_entries);

    list.append(&row);
}
