use gtk4::ListBox;
use lazy_static::lazy_static;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::daemon::{
    renderer::{app::AppContext, config::AppConf},
    structs::{DaemonCmd, DaemonEvt, Dvoty},
};

use std::{cell::RefMut, collections::HashMap, sync::Arc};

use super::{class::adjust_class, entry::DvotyUIEntry, DvotyEntry};

struct Letter {
    pub uppercase: String,
    pub lowercase: String,
}

fn letter(a: &str, b: &str) -> Letter {
    Letter {
        uppercase: a.to_string(),
        lowercase: b.to_string(),
    }
}

lazy_static! {
    static ref LETTERS: HashMap<String, Vec<Letter>> = {
        let mut map = HashMap::new();

        // Helper function to insert a single letter
        let mut insert_letter = |key: &str, uppercase: &str, lowercase: &str| {
            map.entry(key.to_string())
                .or_insert_with(Vec::new)
                .push(letter(uppercase, lowercase));
        };

        // Greek letters
        let greek_letters = [
            ("alpha", "Α", "α"), ("beta", "Β", "β"), ("gamma", "Γ", "γ"),
            ("delta", "Δ", "δ"), ("epsilon", "Ε", "ε"), ("zeta", "Ζ", "ζ"),
            ("eta", "Η", "η"), ("theta", "Θ", "θ"), ("iota", "Ι", "ι"),
            ("kappa", "Κ", "κ"), ("lambda", "Λ", "λ"), ("mu", "Μ", "μ"),
            ("nu", "Ν", "ν"), ("xi", "Ξ", "ξ"), ("omicron", "Ο", "ο"),
            ("pi", "Π", "π"), ("rho", "Ρ", "ρ"), ("sigma", "Σ", "σ"),
            ("tau", "Τ", "τ"), ("upsilon", "Υ", "υ"), ("phi", "Φ", "φ"),
            ("chi", "Χ", "χ"), ("psi", "Ψ", "ψ"), ("omega", "Ω", "ω"),
        ];

        for (name, upper, lower) in greek_letters.iter() {
            insert_letter(name, upper, lower);
        }

        // Nordic letters
        let nordic_letters = [
            ("a-ring", "Å", "å"), ("a with ring", "Å", "å"), ("a with a ring", "Å", "å"),
            ("a-dot", "Ä", "ä"), ("a with dots", "Ä", "ä"), ("a with two dots", "Ä", "ä"),
            ("o-dot", "Ö", "ö"), ("o with dots", "Ö", "ö"), ("o with two dots", "Ö", "ö"),
            ("ae", "Æ", "æ"), ("o-slash", "Ø", "ø"), ("o with slash", "Ø", "ø"), ("o with a slash", "Ø", "ø"),
            ("thorn", "Þ", "þ"),
            ("eth", "Ð", "ð"),
        ];

        for (name, upper, lower) in nordic_letters.iter() {
            insert_letter(name, upper, lower);
        }

        // Extra symbols
        insert_letter("int", "∫", "∫");

        map
    };
}

fn search_letter(kwd: &str, mode: &[bool; 2]) -> Option<Vec<String>> {
    if let Some(val) = LETTERS.get(kwd) {
        return Some(
            val.iter()
                .map(|letter| {
                    let mut result = vec![];
                    if mode[0] {
                        result.push(letter.uppercase.clone());
                    }

                    if mode[1] {
                        result.push(letter.lowercase.clone());
                    }

                    result
                })
                .flatten()
                .collect(),
        );
    }

    None
}

pub fn process_greek_letters(input: String, sender: UnboundedSender<DaemonEvt>, id: &Uuid) {
    // [uppercase, lowercase]
    let mut modes: [bool; 2] = [true, true];
    let mut should_cut: bool = false;

    match input.chars().next() {
        Some(ref ch) => match ch {
            '+' => {
                modes[1] = false;
                should_cut = true;
            }
            '-' => {
                modes[0] = false;
                should_cut = true;
            }
            _ => {}
        },
        None => {
            return;
        }
    }

    let input = if should_cut {
        input.chars().skip(1).collect::<String>().to_lowercase()
    } else {
        input.to_lowercase()
    };

    if let Some(val) = search_letter(&input, &modes) {
        val.iter().for_each(|v| {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Letter {
                        letter: v.clone(),
                    })),
                    sender: None,
                    uuid: Some(*id),
                })
                .unwrap_or_else(|e| {
                    println!("Dvoty: can't send letter: {}", e);
                })
        });
    }
}

pub fn populate_letter_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    letter: String,
    context: &mut RefMut<AppContext>,
    sender: UnboundedSender<DaemonEvt>,
) {
    let row = super::entry::create_base_entry(
        &config.dvoty.letter_icon,
        &letter,
        "Click to copy",
        sender,
    );

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Letter { letter }, row.clone()));

    if context.dvoty.dvoty_entries.len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty.dvoty_entries);
    }

    list.append(&row);
}
