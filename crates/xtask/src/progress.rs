//! Vérification de cohérence de `PROGRESS.json` (T0.1.8).
//!
//! Règles refusées : plus d'une tâche `IN_PROGRESS` ; doublon dans `order` ;
//! tâche `DONE`/`IN_PROGRESS` dont une dépendance déclarée n'est pas `DONE`
//! (dépendance inconnue comprise) ; dépendance placée après sa tâche dans
//! `order` ; `current_task` incohérent (doit pointer la tâche unique
//! `IN_PROGRESS`, ou être `null` s'il n'y en a aucune).

use crate::workspace_root;
use std::collections::HashSet;
use std::path::PathBuf;

/// Valeur JSON minimale (xtask n'a pas de dépendance externe autorisée).
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        let Json::Obj(p) = self else {
            return None;
        };
        p.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
}

// --- Parseur JSON minimal --------------------------------------------------

fn parse(src: &str) -> Result<Json, String> {
    let b = src.as_bytes();
    let mut i = 0;
    let v = value(b, &mut i)?;
    skip_ws(b, &mut i);
    if i != b.len() {
        return Err(format!("caractères en trop à la position {i}"));
    }
    Ok(v)
}

fn skip_ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && matches!(b[*i], b' ' | b'\t' | b'\n' | b'\r') {
        *i += 1;
    }
}

fn value(b: &[u8], i: &mut usize) -> Result<Json, String> {
    skip_ws(b, i);
    match b.get(*i).ok_or("fin d'entrée inattendue")? {
        b'{' => object(b, i),
        b'[' => array(b, i),
        b'"' => Ok(Json::Str(string(b, i)?)),
        _ if b[*i..].starts_with(b"true") => {
            *i += 4;
            Ok(Json::Bool(true))
        }
        _ if b[*i..].starts_with(b"false") => {
            *i += 5;
            Ok(Json::Bool(false))
        }
        _ if b[*i..].starts_with(b"null") => {
            *i += 4;
            Ok(Json::Null)
        }
        _ => number(b, i),
    }
}

fn object(b: &[u8], i: &mut usize) -> Result<Json, String> {
    *i += 1; // '{'
    let mut out = Vec::new();
    skip_ws(b, i);
    if b.get(*i) == Some(&b'}') {
        *i += 1;
        return Ok(Json::Obj(out));
    }
    loop {
        let key = string(b, i)?;
        skip_ws(b, i);
        if b.get(*i) != Some(&b':') {
            return Err(format!("':' attendu à la position {i}"));
        }
        *i += 1;
        out.push((key, value(b, i)?));
        skip_ws(b, i);
        match b.get(*i).ok_or("objet non terminé")? {
            b',' => {
                *i += 1;
                skip_ws(b, i);
            }
            b'}' => {
                *i += 1;
                return Ok(Json::Obj(out));
            }
            _ => {
                eprintln!(
                    "OBJ-ERR at pos {} byte {:?} (last key was {:?})",
                    i,
                    b.get(*i),
                    out.last().map(|(k, _)| k.as_str())
                );
                return Err(format!("',' ou '}}' attendu à la position {i}"));
            }
        }
    }
}

fn array(b: &[u8], i: &mut usize) -> Result<Json, String> {
    *i += 1; // '['
    let mut out = Vec::new();
    skip_ws(b, i);
    if b.get(*i) == Some(&b']') {
        *i += 1;
        return Ok(Json::Arr(out));
    }
    loop {
        out.push(value(b, i)?);
        skip_ws(b, i);
        match b.get(*i).ok_or("tableau non terminé")? {
            b',' => {
                *i += 1;
                skip_ws(b, i);
            }
            b']' => {
                *i += 1;
                return Ok(Json::Arr(out));
            }
            _ => return Err(format!("',' ou ']' attendu à la position {i}")),
        }
    }
}

fn string(b: &[u8], i: &mut usize) -> Result<String, String> {
    // `i` pointe sur le guillemet ouvrant.
    if b.get(*i).copied() != Some(b'"') {
        return Err("guillemet attendu".into());
    }
    *i += 1; // passer le guillemet ouvrant
    let start = *i;
    let text = std::str::from_utf8(&b[start..]).unwrap_or_default();
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '"' => {
                *i += c.len_utf8(); // avancer au-delà du guillemet fermant (1 pour l'ASCII)
                return Ok(out);
            }
            '\\' => {
                // Backslash : consommer le backslash puis le caractère échappé.
                let next = text[*i + 1..].chars().next().ok_or("échappement incomplet")?;
                out.push(match next {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'n' => '\n',
                    't' => '\t',
                    _ => return Err(format!("échappement \\{next} non supporté")),
                });
                *i += 1 + next.len_utf8(); // backslash + caractère échappé
            }
            c => {
                out.push(c);
                *i += c.len_utf8(); // avancer du bon nombre de bytes (multioctets compris)
            }
        }
    }
    Err("chaîne non terminée".into())
}

fn number(b: &[u8], i: &mut usize) -> Result<Json, String> {
    let start = *i;
    while let Some(c) = b.get(*i) {
        if matches!(c, b'0'..=b'9' | b'-' | b'.' | b'e' | b'E' | b'+') {
            *i += 1;
        } else {
            break;
        }
    }
    let s = std::str::from_utf8(&b[start..*i]).map_err(|_| "nombre invalide".to_owned())?;
    if s.is_empty() {
        return Err(format!("valeur attendue à la position {start}"));
    }
    s.parse::<f64>().map(Json::Num).map_err(|e| e.to_string())
}

// --- Règles de cohérence ---------------------------------------------------

fn deps_of(t: &Json) -> Vec<&Json> {
    t.get("deps")
        .and_then(|d| match d {
            Json::Arr(a) => Some(a),
            _ => None,
        })
        .map_or_else(Vec::new, |a| a.iter().collect())
}

/// Vérifie `PROGRESS.json` (chemin par défaut : racine du workspace).
pub fn check(path: Option<&str>) -> Result<(), Vec<String>> {
    let file = match path {
        Some(p) => PathBuf::from(p),
        None => workspace_root().join("PROGRESS.json"),
    };
    let doc = std::fs::read_to_string(&file)
        .map_err(|e| vec![format!("impossible de lire {} : {e}", file.display())])?;
    check_doc(&doc)
}

/// Statut d'une tâche (`tasks.<id>.status`), si présent et string.
fn status_of(t: &Json) -> Option<&str> {
    t.get("status").and_then(Json::as_str)
}

/// Vérifie les règles de cohérence sur un document `PROGRESS.json`.
pub fn check_doc(doc: &str) -> Result<(), Vec<String>> {
    let root = match parse(doc) {
        Ok(v) => v,
        Err(e) => return Err(vec![format!("JSON invalide : {e}")]),
    };
    let order = root
        .get("order")
        .and_then(|v| match v {
            Json::Arr(a) if a.iter().all(|x| x.as_str().is_some()) => Some(a.clone()),
            _ => None,
        })
        .ok_or_else(|| vec!["champ `order` manquant ou invalide (tableau d'IDs de tâches)".to_owned()])?;
    let tasks = root
        .get("tasks")
        .and_then(|v| match v {
            Json::Obj(o) => Some(o.clone()),
            _ => None,
        })
        .ok_or_else(|| vec!["champ `tasks` manquant ou invalide".to_owned()])?;

    let mut errors = Vec::new();

    // Règle 1 : au plus une tâche IN_PROGRESS.
    let in_progress: Vec<&String> = tasks
        .iter()
        .filter(|(_, t)| status_of(t) == Some("IN_PROGRESS"))
        .map(|(id, _)| id)
        .collect();
    if in_progress.len() > 1 {
        errors.push(format!(
            "plus d'une tâche IN_PROGRESS : {}",
            in_progress.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    // Règle 2 : pas de doublon dans `order`.
    let mut seen = HashSet::new();
    for id in &order {
        let Some(id) = id.as_str() else {
            continue;
        };
        if !seen.insert(id) {
            errors.push(format!("doublon dans `order` : {id}"));
        }
    }

    // Règle 3 : une tâche DONE/IN_PROGRESS a toutes ses dépendances déclarées DONE.
    for (id, t) in &tasks {
        if matches!(status_of(t), Some("DONE" | "IN_PROGRESS")) {
            for dep in deps_of(t) {
                let Some(dep) = dep.as_str() else {
                    continue;
                };
                match tasks.iter().find(|(k, _)| k == dep) {
                    None => errors.push(format!("tâche {id} : dépendance inconnue {dep}")),
                    Some((_, dt)) if status_of(dt) != Some("DONE") => {
                        let st = status_of(dt).unwrap_or("<absent>");
                        errors.push(format!("tâche {id} : dépendance {dep} non DONE (statut : {st})"));
                    }
                    _ => {}
                }
            }
        }
    }

    // Règle 4 : aucune dépendance placée après sa tâche dans `order`.
    for (id, t) in &tasks {
        let Some(pi) = order.iter().position(|o| o.as_str() == Some(id)) else {
            continue;
        };
        for dep in deps_of(t) {
            let Some(dep) = dep.as_str() else {
                continue;
            };
            if let Some(pd) = order.iter().position(|o| o.as_str() == Some(dep)) {
                if pd > pi {
                    errors.push(format!("dépendance {dep} placée après sa tâche {id} dans `order`"));
                }
            }
        }
    }

    // Règle 5 : current_task cohérent avec les tâches IN_PROGRESS.
    match root.get("current_task") {
        None | Some(Json::Null) => {
            if !in_progress.is_empty() {
                errors.push(format!(
                    "current_task est null mais tâche(s) IN_PROGRESS : {}",
                    in_progress.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                ));
            }
        }
        Some(Json::Str(id)) => match tasks.iter().find(|(k, _)| k == id) {
            None => errors.push(format!("current_task pointe vers la tâche inconnue {id}")),
            Some((_, t)) if status_of(t) != Some("IN_PROGRESS") => {
                let st = status_of(t).unwrap_or("<absent>");
                errors.push(format!("current_task = {id} mais la tâche n'est pas IN_PROGRESS (statut : {st})"));
            }
            _ => {}
        },
        Some(_) => errors.push("current_task doit être null ou un ID de tâche".to_owned()),
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fabrique un document minimal : `order`, `current_task`, tâches (id, statut, dépendance).
    fn fixture(order: &[&str], tasks: &[(&str, &str, Option<&str>)], current_task: Option<&str>) -> String {
        let t = tasks
            .iter()
            .map(|(id, st, dep)| {
                let deps = match dep {
                    Some(d) => format!("\"{d}\""),
                    None => String::new(),
                };
                format!(
                    "\"{id}\":{{\"title\":\"t\",\"deps\":[{deps}],\"status\":\"{st}\",\"attempts\":0,\"commit\":null,\"report\":null,\"notes\":[]}}"
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let o = order.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(",");
        let c = current_task.map_or_else(|| "null".to_owned(), |c| format!("\"{c}\""));
        // Build: {"order":[...],"current_task":...,"halted":false,"tasks":{...},"kb_gaps":[]}
        format!(r#"{{"order":[{o}],"current_task":{c},"halted":false,"tasks":{{{t}}},"kb_gaps":[] }}"#)
    }

    #[test]
    fn valide_pass() {
        let doc = fixture(
            &["T1", "T2"],
            &[("T1", "DONE", None), ("T2", "TODO", Some("T1"))],
            None,
        );
        assert!(check_doc(&doc).is_ok());
    }

    #[test]
    fn debug_valide_pass() {
        let doc = fixture(
            &["T1", "T2"],
            &[("T1", "DONE", None), ("T2", "TODO", Some("T1"))],
            None,
        );
        println!("VALIDE_PASS DOC:\n{doc}\n---END---");
        match parse(&doc) {
            Ok(v) => println!("parse OK: {v:?}"),
            Err(e) => panic!("parse failed for valide_pass fixture: {e}"),
        }
        assert!(check_doc(&doc).is_ok());
    }

    #[test]
    fn deux_in_progress_refuse() {
        let doc = fixture(
            &["T1", "T2"],
            &[("T1", "IN_PROGRESS", None), ("T2", "IN_PROGRESS", None)],
            Some("T1"),
        );
        assert!(check_doc(&doc).is_err());
    }

    #[test]
    fn doublon_order_refuse() {
        let doc = fixture(
            &["T1", "T2", "T1"],
            &[("T1", "TODO", None), ("T2", "TODO", None)],
            None,
        );
        assert!(check_doc(&doc).is_err());
    }

    #[test]
    fn done_dep_non_done_refuse() {
        let doc = fixture(
            &["T1", "T2"],
            &[("T1", "DONE", Some("T2")), ("T2", "TODO", None)],
            None,
        );
        assert!(check_doc(&doc).is_err());
    }

    #[test]
    fn dep_apres_son_tache_refuse() {
        let doc = fixture(
            &["T1", "T2"],
            &[("T1", "TODO", Some("T2")), ("T2", "DONE", None)],
            None,
        );
        assert!(check_doc(&doc).is_err());
    }

    #[test]
    fn current_task_incoherent_refuse() {
        let doc = fixture(
            &["T1"],
            &[("T1", "TODO", None)],
            Some("T1"), // T1 n'est pas IN_PROGRESS
        );
        assert!(check_doc(&doc).is_err());
    }

    #[test]
    fn progress_json_reel_valide() {
        match check(None) {
            Ok(()) => {}
            Err(e) => panic!("real PROGRESS.json rejected: {e:?}"),
        }
    }

    #[test]
    fn debug_parse_doc() {
        let doc = r#"{"order":["T1","T2"],"current_task":null,"halted":false,"tasks":{"T1":{"title":"t","deps":[],"ctx":[],"status":"DONE","attempts":0,"commit":null,"report":null,"notes":[]},"T2":{"title":"t","deps":["T1"],"ctx":[],"status":"TODO","attempts":0,"commit":null,"report":null,"notes":[]}},"kb_gaps":[]}"#;
        match parse(doc) {
            Ok(v) => println!("parsed: {v:?}"),
            Err(e) => panic!("parse failed: {e}"),
        }
    }

    #[test]
    fn debug_bisect_real() {
        let s = std::fs::read_to_string(crate::workspace_root().join("PROGRESS.json")).unwrap();
        for n in 1..=s.len() {
            if parse(&s[..n]).is_err() {
                println!("first-fail at byte {n}");
                // Show character-boundary context instead of raw bytes.
                let start = s[..n]
                    .rfind(|c: char| c.is_ascii() || !c.is_control())
                    .unwrap_or(0);
                let end = s[n..]
                    .find(|c: char| c as u32 > 32 || c == '\n' || c == '\t')
                    .map_or(s.len(), |p| n + p);
                println!(
                    "ctx: {:?}",
                    &s[start.max(n.saturating_sub(60)).min(n.min(s.len()))..end]
                );
                return;
            }
        }
        println!("no prefix failed");
    }
}
