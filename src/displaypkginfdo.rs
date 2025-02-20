use colored::Colorize;
use comfy_table::{Table, Row, Cell, presets::UTF8_FULL};
pub fn display_package_info(pkg: &(String, crate::parse_pkg_index::Package)) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    
    // Set the table header
    table.set_header(vec![
        Cell::new("Field").set_alignment(comfy_table::CellAlignment::Center),
        Cell::new("Value").set_alignment(comfy_table::CellAlignment::Center),
    ]);

    // Determine GUI support display
    let supports_gui = if pkg.1.gui {
        "Yes".green().bold().to_string()
    } else {
        "No".red().bold().to_string()
    };

    // Join binary paths and symlink names for a cleaner display
    let binary_locations = pkg.1.binary_at.join("\n");
    let symlink_names = pkg.1.symlink_names.join(", ");

    // Add rows with package information
    table.add_row(Row::from(vec![
        Cell::new("Name"),
        Cell::new(pkg.0.green().bold().to_string()),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("Version"),
        Cell::new(pkg.1.version.green().bold().to_string()),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("Description"),
        Cell::new(pkg.1.description.green().bold().to_string()),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("Supports GUI"),
        Cell::new(supports_gui),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("Binary Located at"),
        Cell::new(binary_locations.green().bold().to_string()),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("Symlink it creates"),
        Cell::new(symlink_names.green().bold().to_string()),
    ]));
    table.add_row(Row::from(vec![
        Cell::new("Icon URL"),
        Cell::new(pkg.1.icon_at.green().bold().to_string()),
    ]));

    // Print the final table
    println!("{}", table);
}