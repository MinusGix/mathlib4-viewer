use std::{
    path::Path,
    sync::{Arc, Mutex},
};

// use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::js_iter::js_iter_order;

// TODO: these could be borrowed strings?
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclData {
    #[serde(default)]
    pub declarations: IndexMap<Arc<str>, Decl>,
    #[serde(default)]
    pub instances: IndexMap<Arc<str>, Vec<String>>,
    #[serde(default)]
    pub instances_for: IndexMap<Arc<str>, Vec<String>>,
    #[serde(default)]
    pub imports: Vec<String>,
    #[serde(default)]
    pub imported_by: IndexMap<Arc<str>, Vec<String>>,
    #[serde(default)]
    pub modules: IndexMap<Arc<str>, String>,

    /// Cached declarations iter order that matches the JavaScript iteration order.
    #[serde(skip)]
    declarations_iter_order: Mutex<Option<Arc<Vec<Arc<str>>>>>,
}
impl DeclData {
    fn decl_iter_order(&self) -> Arc<Vec<Arc<str>>> {
        let mut res = self.declarations_iter_order.lock().unwrap();
        if res.is_none() {
            let order = js_iter_order(&self.declarations);

            res.replace(Arc::new(order));
        }

        res.clone().unwrap()
    }

    pub fn search_strict<'s>(&'s self, text: &str) -> Option<&'s Decl> {
        self.declarations.get(text)
    }

    /// Search for a specific declaration
    pub fn search<'s>(
        &'s self,
        pattern: &str,
        allowed_kinds: Option<&[DeclKind]>,
        max_results: Option<usize>,
    ) -> Vec<&'s Decl> {
        // Use ascii lowercase to avoid weird things with unicode symbols used for math.
        let lower_patterns = pattern.to_ascii_lowercase();
        let mut lower_patterns = lower_patterns.split_whitespace();

        let pattern_no_spaces = pattern.split_whitespace().collect::<String>();

        let mut results = Vec::new();

        // for decl in self.declarations.values() {
        let keys = self.decl_iter_order();
        for key in keys.iter() {
            let decl = self.declarations.get(key).unwrap();

            if let Some(allowed_kinds) = allowed_kinds {
                if !allowed_kinds.contains(&decl.kind) {
                    continue;
                }
            }

            // let lower_name = decl.name.to_ascii_lowercase();
            let lower_doc = decl.doc.to_ascii_lowercase();

            // TODO: we could just skip spaces in match case sensitive rather than allocating a string
            let Some(mut err) = match_case_sensitive(&decl.name, &pattern_no_spaces) else {
                continue;
            };

            // Match all words as substrings of docstring
            if err >= 3.0 && pattern.len() > 3 {
                let lower_doc_contains_pat = lower_patterns.all(|x| lower_doc.contains(x));

                if lower_doc_contains_pat {
                    err = 3.0;
                }
            }

            results.push((err, decl));
        }

        // Sort by score which is an f64 so no ord but we just assume no nan/infinities
        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Take only the top `max_results` results
        if let Some(max_results) = max_results {
            results.truncate(max_results);
        }

        // Return only the declarations
        results.into_iter().map(|(_, decl)| decl).collect()

        // let mut results = Vec::new();

        // let matcher = SkimMatcherV2::default();

        // for decl in self.declarations.values() {
        //     if let Some(allowed_kinds) = allowed_kinds {
        //         if !allowed_kinds.contains(&decl.kind) {
        //             continue;
        //         }
        //     }

        //     if let Some(score) = matcher.fuzzy_match(&decl.name, pattern) {
        //         // TODO: check if it is close enough??
        //         results.push((score, decl));
        //     }
        // }

        // // Sort by score
        // results.sort_by(|a, b| b.0.cmp(&a.0));

        // // Take only the top `max_results` results
        // if let Some(max_results) = max_results {
        //     results.truncate(max_results);
        // }

        // // Return only the declarations
        // results.into_iter().map(|(_, decl)| decl).collect()
    }
}

fn match_case_sensitive(decl_name: &str, pattern: &str) -> Option<f64> {
    let mut err = 0.0;
    let mut last_match = 0;

    let diter = decl_name.char_indices();
    let mut piter = pattern.chars().peekable();

    for (didx, d) in diter {
        let Some(p) = piter.peek() else {
            break;
        };
        let p = *p;

        if p.eq_ignore_ascii_case(&d) {
            let diff = didx - last_match;
            err += if is_separator(p) {
                0.125 * diff as f64
            } else {
                diff as f64
            };

            // If they aren't the same case
            if p != d {
                err += 0.5;
            }

            last_match = didx + d.len_utf8();
            piter.next();
        } else if is_separator(d) {
            err += 0.125 * (didx + d.len_utf8() - last_match) as f64;
            last_match = didx + d.len_utf8();
        }
    }

    err += 0.125 * (decl_name.len() - last_match) as f64;

    if piter.peek() == None {
        // We reached the end of the pattern
        Some(err)
    } else {
        None
    }
}

fn is_separator(ch: char) -> bool {
    ch == '.' || ch == '_'
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Decl {
    pub source_link: String,
    pub name: String,
    pub kind: DeclKind,
    pub doc_link: String,
    pub doc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeclKind {
    Ctor,
    Def,
    Instance,
    Theorem,
    Axiom,
    Inductive,
    Structure,
    Class,
    Opaque,
}

/// Deserialize the declaration data from the given path.
pub fn load_decl_data(path: &Path) -> Result<DeclData, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    let data = serde_json::from_str(&data)?;
    Ok(data)
}
