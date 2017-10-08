extern crate byteorder;

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, stdout, Write, Cursor};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt, LittleEndian};
use std::collections::HashMap;

fn parse_data(map: &mut HashMap<u16, String>) {
    let data = include_str!("data");
    for line in data.split("\n") {
        let vec: Vec<&str> = line.trim().split(": ").collect();
        if vec.len() >= 2 {
            //println!("{} {}", vec[0], vec[1]);
            map.insert(vec[1].parse().unwrap(), vec[0].to_string());
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut name_map = HashMap::new();
    parse_data(&mut name_map);
    if args.len() < 2 {
        println!("Please select a save file");
        std::process::exit(1);
    }
    let filename = &args[1];
    let mut save_file = File::open(filename).unwrap();
    let mut buf = vec![];
    save_file.read_to_end(&mut buf).unwrap();

    let stdin = std::io::stdin();

    loop {
        print!("> ");
        stdout().flush().unwrap();
        let line = stdin
            .lock()
            .lines()
            .next()
            .expect("there was no next line")
            .expect("the line could not be read");
        match line.as_str() {
            "help" => {
                println!("RTFM");
            },
            "list" => {
                let mut offset = 0x1dc;
                loop {
                    //Inventory ends at 0x383
                    if offset > 0x383 {
                        break;
                    }
                    let mut idv = Cursor::new(vec![buf[offset], buf[offset + 1]]);
                    let id = idv.read_u16::<LittleEndian>().unwrap();
                    let mut amount_slice = Cursor::new(&buf[offset + 2..offset + 6]);
                    let amount = amount_slice.read_u32::<LittleEndian>().unwrap();
                    println!("Slot: {} ID: 0x{:x}, Amount: {} Name: {}", (offset - 0x1dc) / 12, id, amount, name_map[&id]);
                    offset += 12;
                }
            },
            "edit" => {
                    println!("Which slot?");
                    let line = stdin
                            .lock()
                            .lines()
                            .next()
                            .expect("there was no next line")
                            .expect("the line could not be read");
                    let slot: usize = match line.parse() {
                        Ok(num) => num,
                        Err(_) => {
                            println!("Please input a valid number");
                            continue;
                        }
                    };
                    if slot > 35 {
                        println!("There is no slot {}", slot);
                        continue;
                    }
                    println!("Input the name of the item or its id in hexadecimal");
                    let line = stdin
                            .lock()
                            .lines()
                            .next()
                            .expect("there was no next line")
                            .expect("the line could not be read");
                    let mut number = true;
                    let num = match u16::from_str_radix(line.as_str(), 16) {
                        Ok(v) => v,
                        Err(_) => {
                            number = false;
                            0
                        }
                    };
                    let mut found = false;
                    let mut id: u16 = 0;
                    if number {
                        println!("The item you selected is: {}, please enter the amount:", name_map[&num]);
                        id = match name_map.get(&num) {
                            Some(_) => {
                                found = true;
                                num
                            },
                            None => {
                                found = false;
                                0
                            }
                        };
                    } else {
                        for (n, name) in name_map.iter() {
                            let name = name.replace("\"", "").to_string();
                            if name == line.trim() || name.to_lowercase() == line.trim().to_lowercase() {
                                println!("The item you selected is: {}, please enter the amount:", name_map[n]);
                                id = *n;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        println!("No such item");
                        continue;
                    }
                    let mut amount;
                    loop {
                        let stop;
                        let line = stdin
                            .lock()
                            .lines()
                            .next()
                            .expect("there was no next line")
                            .expect("the line could not be read");

                        amount = match line.parse() {
                            Ok(num) => {
                                stop = true;
                                num
                            },
                            Err(_) => {
                                println!("Please input a number");
                                continue;
                            }
                        };
                        if stop {
                            break;
                        }
                    }
                    let mut id_write = vec![];
                    id_write.write_u16::<BigEndian>(id).unwrap();
                    let mut amount_write = vec![];
                    amount_write.write_u32::<LittleEndian>(amount).unwrap();
                    //Write the id
                    buf[((0x1dc + (slot * 12)))] = id_write[1];
                    buf[((0x1dc + (slot * 12) + 1))] = id_write[0];
                    //Write the amount
                    buf[((0x1dc + (slot * 12) + 2))] = amount_write[0];
                    buf[((0x1dc + (slot * 12) + 3))] = amount_write[1];
                    buf[((0x1dc + (slot * 12) + 4))] = amount_write[2];
                    buf[((0x1dc + (slot * 12) + 5))] = amount_write[3];
                    let mut save_file = File::create(filename).unwrap();
                    save_file.write_all(&buf).unwrap();
            },
            "exit" => {
                std::process::exit(1);
            }
            _ => {
                println!("No such command: {}", line)
            }
        }
    }
}
