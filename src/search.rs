use std::{collections::HashMap, process::exit};
use colored::Colorize;
use crate::parse_pkg_index::{self, Package};

pub fn searchpkg(search_terms: &Vec<String>, index: &String) -> HashMap<String, Package> {
    let pkg_index = parse_pkg_index::ppkgi(index);
    
    let mut missing = Vec::new();

    for term in search_terms {
        if !pkg_index.contains_key(term) {
            missing.push(term.clone());
        }
    }

    if !missing.is_empty() {
        println!("{}", "┌[Following packages were not found!]".red().bold());
        for term in &missing {
            println!("├─ {}", term.red().bold());
        }
        println!("{}", "└[Please look online for correct package names or contact us!]".red().bold());
        exit(1);
    }
    pkg_index
}
