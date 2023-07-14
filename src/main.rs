use ipnetwork::IpNetwork;
use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::net::TcpStream;
use std::time::Duration;
use colored::*;



#[derive(Debug, Serialize, Deserialize, Clone)]
struct IpInfo {
    id: usize,
    ip: String,
    subnet: String,
    gateway: String,
    description: String,
    port: u16,
}

fn is_online(ip: &str, port: u16) -> bool {
    let address = format!("{}:{}", ip, port);
    match TcpStream::connect_timeout(&address.parse().unwrap(), Duration::from_secs(1)) {
        Ok(_) => true,
        Err(_) => false,
    }
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
        Cell::new("ID"),
        Cell::new("IP"),
        Cell::new("Subnet"),
        Cell::new("Gateway"),
        Cell::new("Description"),
        Cell::new("Port"),
        Cell::new("Status"),
    ]));

    for info in ip_info {
        let online = is_online(&info.ip, info.port);
        let status = if online { "Online".green() } else { "Offline".red() };

        table.add_row(Row::new(vec![
            Cell::new(&info.id.to_string()),
            Cell::new(&info.ip),
            Cell::new(&info.subnet),
            Cell::new(&info.gateway),
            Cell::new(&info.description),
            Cell::new(&info.port.to_string()),
            Cell::new(&status.to_string()),
        ]));
    }

    table.printstd();
}


fn add_ip(ip_info: &mut Vec<IpInfo>) {
    let ip = prompt("Enter IP address:");
    let subnet = prompt("Enter subnet mask:");
    let gateway = prompt("Enter gateway:");
    let description = prompt("Enter description:");
    let port = prompt("Enter port:");

    let port = match port.parse::<u16>() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Invalid port number! IP information not added.");
            return;
        }
    };

    if validate_ip(&ip) && validate_subnet(&subnet) && validate_gateway(&gateway) {
        let new_ip = IpInfo {
            id: ip_info.len() + 1, // Adding 1 here
            ip,
            subnet,
            gateway,
            description,
            port,
        };
        ip_info.push(new_ip);
        println!("IP information added successfully!");
    } else {
        eprintln!("Invalid IP, subnet, or gateway! IP information not added.");
    }
}


fn delete_ip(ip_info: &mut Vec<IpInfo>) {
    let id_to_delete = prompt("Enter ID to delete:");

    let id_to_delete = match id_to_delete.parse::<usize>() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Invalid ID! Deletion failed.");
            return;
        }
    };

    if id_to_delete == 0 || id_to_delete > ip_info.len() {
        eprintln!("ID out of range! Deletion failed.");
        return;
    }

    ip_info.remove(id_to_delete - 1); // Subtracting 1 here
    println!("IP information deleted successfully!");
}

fn edit_ip(ip_info: &mut Vec<IpInfo>) {
    let id_to_edit = prompt("Enter ID to edit:");

    let id_to_edit = match id_to_edit.parse::<usize>() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Invalid ID! Edit failed.");
            return;
        }
    };

    if id_to_edit == 0 || id_to_edit > ip_info.len() {
        eprintln!("ID out of range! Edit failed.");
        return;
    }

    let mut updated_ip_info = ip_info[id_to_edit - 1].clone();
    let ip = prompt("Enter new IP address:");
    if !ip.is_empty() {
        if validate_ip(&ip) {
            updated_ip_info.ip = ip;
        } else {
            eprintln!("Invalid IP address! IP information not updated.");
            return;
        }
    }
    let subnet = prompt("Enter new subnet mask:");
    if !subnet.is_empty() {
        updated_ip_info.subnet = subnet;
    }
    let gateway = prompt("Enter new gateway:");
    if !gateway.is_empty() {
        updated_ip_info.gateway = gateway;
    }
    let description = prompt("Enter new description:");
    if !description.is_empty() {
        updated_ip_info.description = description;
    }
    let port = prompt("Enter new port:");
    if !port.is_empty() {
        match port.parse::<u16>() {
            Ok(p) => {
                updated_ip_info.port = p;
            }
            Err(_) => {
                eprintln!("Invalid port number! IP information not updated.");
                return;
            }
        }
    }

    if validate_subnet(&updated_ip_info.subnet) && validate_gateway(&updated_ip_info.gateway) {
        ip_info[id_to_edit - 1] = updated_ip_info;
        println!("IP information updated successfully!");
    } else {
        eprintln!("Invalid subnet or gateway! IP information not updated.");
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

        let choice_result: Result<u32, _> = prompt("Enter your choice:").parse();

        match choice_result {
            Ok(choice) => {
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
                        return;
                    }
                    _ => println!("Invalid choice. Please try again."),
                }
            }
            Err(_) => {
                println!("Invalid choice. Please enter a valid number.");
            }
        }

        println!();

        // Pause and wait for any key to be pressed
        println!("Press Enter to continue...");
        let mut buffer = String::new();
        let _ = io::stdin().read_line(&mut buffer);
    }
}
