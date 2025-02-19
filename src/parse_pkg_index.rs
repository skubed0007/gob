use std::collections::HashMap;

#[derive(Debug,Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub depends: Vec<String>,
    pub url: String,
    pub binary_at: Vec<String>,
    pub symlink_names: Vec<String>,
    pub gui : bool,
    pub icon_at : String,
    pub extractable : bool,
}

pub fn ppkgi(index: &String) -> HashMap<String, Package> {
    let mut packages = HashMap::new();
    let mut current_block: Vec<String> = Vec::new();
    let mut in_block = false;

    for line in index.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('{') {
            in_block = true;
            current_block.clear();
            continue;
        }
        if line.starts_with('}') {
            in_block = false;
            let mut name = String::new();
            let mut version = String::new();
            let mut description = String::new();
            let mut depends = Vec::new();
            let mut url = String::new();
            let mut binary_at = Vec::new();
            let mut symlink_names = Vec::new();
            let mut gui = false;
            let mut icon_at = String::new();
            let mut eable = true;

            for block_line in &current_block {
                if let Some(pos) = block_line.find('=') {
                    let key = block_line[..pos].trim();
                    let value = block_line[pos + 1..].trim();
                    match key {
                        "name" => name = value.to_string(),
                        "version" => version = value.to_string(),
                        "description" => description = value.to_string(),
                        "depends" => {
                            if !value.is_empty() {
                                depends = value
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                        }
                        "url" => url = value.to_string(),
                        "binary-at" => {
                            if !value.is_empty() {
                                binary_at = value
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                        }
                        "symlink-names" => {
                            if !value.is_empty() {
                                symlink_names = value
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                        }
                        "desktop" => {
                            if value == true.to_string() {
                                gui = true;
                            }
                        }
                        "icon" => {
                            icon_at = value.to_string();
                        }
                        "extractable" => {
                            if value == false.to_string() {
                                eable = false;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if !name.is_empty() {
                let pkg = Package {
                    name: name.clone(),
                    version,
                    description,
                    depends,
                    url,
                    binary_at,
                    symlink_names,
                    gui,
                    icon_at,
                    extractable: eable,
                };
                packages.insert(name, pkg);
            }
            continue;
        }
        if in_block {
            current_block.push(line.to_string());
        }
    }
    packages
}
