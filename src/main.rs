use ipnetwork::IpNetwork;
use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};


#[derive(Debug, Serialize, Deserialize, Clone)]
struct IpInfo {
    ip: String,
    subnet: String,
    gateway: String,
    description: String,
}


fn validate_ip(ip: &str) -> bool {
    match ip.parse::<IpNetwork>() {
        Ok(IpNetwork::V4(_)) => true,
        _ => false,
    }
}

fn validate_subnet(subnet: &str) -> bool {
    match subnet.parse::<IpNetwork>() {
        Ok(IpNetwork::V4(network)) => network.prefix() <= 32,
        _ => false,
    }
}

fn validate_gateway(gateway: &str) -> bool {
    validate_ip(gateway)
}

fn save_to_database(db: &str, ip_info: &Vec<IpInfo>) -> Result<(), std::io::Error> {
    let json_data = json!(ip_info);

    let json_str = serde_json::to_string_pretty(&json_data)?;

    let mut file = File::create(db)?;
    file.write_all(json_str.as_bytes())?;

    Ok(())
}

fn load_from_database(db: &str) -> Result<Vec<IpInfo>, std::io::Error> {
    let mut file = File::open(db)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let json_data: Vec<IpInfo> = serde_json::from_str(&contents)?;

    Ok(json_data)
}

fn display_table(ip_info: &[IpInfo]) {
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("IP").style_spec("bFg"),
        Cell::new("Subnet").style_spec("bFg"),
        Cell::new("Gateway").style_spec("bFg"),
        Cell::new("Description").style_spec("bFg"),
    ]));
    


    for info in ip_info {
        table.add_row(Row::new(vec![
            Cell::new(&info.ip),
            Cell::new(&info.subnet),
            Cell::new(&info.gateway),
            Cell::new(&info.description),
        ]));
    }

    table.printstd();
}

fn add_ip(ip_info: &mut Vec<IpInfo>) {
    let ip = prompt("Enter IP address:");
    let subnet = prompt("Enter subnet mask:");
    let gateway = prompt("Enter gateway:");
    let description = prompt("Enter description:");

    if validate_ip(&ip) && validate_subnet(&subnet) && validate_gateway(&gateway) {
        let new_ip = IpInfo {
            ip,
            subnet,
            gateway,
            description,
        };
        ip_info.push(new_ip);
        println!("IP information added successfully!");
    } else {
        eprintln!("Invalid IP, subnet, or gateway! IP information not added.");
    }
}

fn delete_ip(ip_info: &mut Vec<IpInfo>) {
    let ip_to_delete = prompt("Enter IP address to delete:");

    let index = ip_info.iter().position(|info| info.ip == ip_to_delete);

    match index {
        Some(idx) => {
            ip_info.remove(idx);
            println!("IP information deleted successfully!");
        }
        None => {
            eprintln!("IP address not found in the database. Deletion failed.");
        }
    }
}

fn edit_ip(ip_info: &mut Vec<IpInfo>) {
    let ip_to_edit = prompt("Enter IP address to edit:");

    let index = ip_info.iter().position(|info| info.ip == ip_to_edit);

    match index {
        Some(idx) => {
            let mut updated_ip_info = ip_info[idx].clone();
            updated_ip_info.subnet = prompt("Enter new subnet mask:");
            updated_ip_info.gateway = prompt("Enter new gateway:");
            updated_ip_info.description = prompt("Enter new description:");

            if validate_subnet(&updated_ip_info.subnet) && validate_gateway(&updated_ip_info.gateway) {
                ip_info[idx] = updated_ip_info;
                println!("IP information updated successfully!");
            } else {
                eprintln!("Invalid subnet or gateway! IP information not updated.");
            }
        }
        None => {
            eprintln!("IP address not found in the database. Edit failed.");
        }
    }
}

fn prompt(message: &str) -> String {
    print!("{} ", message);
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buffer = String::new();
    handle.read_line(&mut buffer).unwrap();

    buffer.trim().to_string()
}

fn main() {
    let db_path = "ip_database.json";

    // Load existing IP information from the database
    let mut ip_info = match load_from_database(db_path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error loading database: {}", err);
            Vec::new()
        }
    };

    loop {
        println!("==== IP Management Menu ====");
        println!("1. Display IP information");
        println!("2. Add IP information");
        println!("3. Delete IP information");
        println!("4. Edit IP information");
        println!("5. Exit");

        let choice: u32 = prompt("Enter your choice:").parse().unwrap();

        match choice {
            1 => display_table(&ip_info),
            2 => add_ip(&mut ip_info),
            3 => delete_ip(&mut ip_info),
            4 => edit_ip(&mut ip_info),
            5 => {
                // Save IP information to the database before exiting
                if let Err(err) = save_to_database(db_path, &ip_info) {
                    eprintln!("Error saving to database: {}", err);
                }
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }

        println!();
    }
}
