use evalexpr::{context_map, EvalexprError, Value};
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::structs::{DaemonCmd, DaemonEvt, Dvoty};

use super::{
    app_launcher::process_apps,
    math::{post_process_result, preprocess_math},
    DvotyEntry,
};

fn identify_math(input: &str) -> Result<Value, EvalexprError> {
    let context = match context_map! {
        "pi" => Value::Float(std::f64::consts::PI),
        "deg" => Function::new(|argument| {
            let arguments = argument.as_number()?;

            Ok(Value::Float(arguments / 180f64 * std::f64::consts::PI))
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
            return Err(e);
        }
    };

    evalexpr::eval_with_context(&preprocess_math(input), &context)
}

pub fn process_general(sender: UnboundedSender<DaemonEvt>, input: &str, id: &uuid::Uuid) {
    // math
    if let Ok(val) = identify_math(input) {
        sender
            .send(DaemonEvt {
                evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Math {
                    expression: input.to_string(),
                    result: post_process_result(val),
                })),
                sender: None,
                uuid: Some(*id),
            })
            .unwrap_or_else(|e| println!("Dvoty: Failed to send math result: {}", e));
    }

    // app launcher
    process_apps(input, sender.clone(), id);

    // search
    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Search {
                keyword: input.into(),
            })),
            sender: None,
            uuid: Some(*id),
        })
        .unwrap_or_else(|e| {
            println!("Dvoty: Error adding search entry: {}", e);
        });
}
