use std::sync::Arc;

use evalexpr::context_map;
use gtk4::{prelude::*, EventControllerKey, GestureClick, ListBox};
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::{
    renderer::{config::AppConf, dvoty::base::DvotyEntry},
    structs::{DaemonCmd, DaemonEvt, Dvoty},
};

fn set_clipboard_text(text: &str) {
    let display = gtk4::gdk::Display::default().expect("Could not get default display");
    let clipboard = display.clipboard();

    clipboard.set_text(text);
}

pub fn eval_math(input: &str, sender: UnboundedSender<DaemonEvt>) {
    use evalexpr::Value;
    let context = match context_map! {
        "pi" => Value::Float(3.1415926),
        "rad" => Function::new(|argument| {
            let arguments = argument.as_number()?;

            Ok(Value::Float(arguments / 180f64 * 3.1415926f64))
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
        "sqrt" => Function::new(|argument| {
            let number = argument.as_number()?;
            if number < 0.0 {
                Err(evalexpr::EvalexprError::CustomMessage("Cannot calculate square root of a negative number".to_string()))
            } else {
                Ok(Value::Float(number.sqrt()))
            }
        })
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
                        result: res.to_string(),
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

pub fn populate_math_entry(config: Arc<AppConf>, list: &ListBox, result: String) {
    let row = super::base::create_base_entry(config, "=", &result, "Click to copy");
    let gesture_click = GestureClick::new();
    let result_clone = result.clone();
    gesture_click.connect_pressed(move |_, _, _, _| {
        set_clipboard_text(&result_clone);
    });

    let key_controller = EventControllerKey::new();

    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Return {
            set_clipboard_text(&result);
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });

    row.add_controller(gesture_click);
    row.add_controller(key_controller);

    list.append(&row);
}
