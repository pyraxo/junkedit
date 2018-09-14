#![feature(rustc_private)]
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

fn write_slot(slot: usize, id: u16, amount: u32, buf: &mut Vec<u8>) {
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

}

fn read_input(prompt: &str) -> String {
    let stdin = std::io::stdin();
    println!("{}", prompt);
    return stdin
            .lock()
            .lines()
            .next()
            .expect("there was no next line")
            .expect("the line could not be read");

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
                    if id != 0xffff {
                        let mut amount_slice = Cursor::new(&buf[offset + 2..offset + 6]);
                        let amount = amount_slice.read_u32::<LittleEndian>().unwrap();
                        println!("Slot: {} ID: 0x{:x}, Amount: {} Name: {}", (offset - 0x1dc) / 12, id, amount, name_map[&id]);
                    }
                    offset += 12;
                }
            },
            "edit" => {
                    let line = read_input("Which slot?");
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
                    let line = read_input("Input the name of the item or its id in hexadecimal");
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
                    write_slot(slot, id, amount, &mut buf);
                    let mut save_file = File::create(filename).unwrap();
                    save_file.write_all(&buf).unwrap();
            },
            "multiedit" => {
                let line = read_input("Input a comma-separated list of item ids:");
                let id_strings: Vec<&str> = line.split(",").collect();
                let mut ids = vec![];
                for id in id_strings {
                    let id = id.trim();
                    let mut number = true;
                    let num = match u16::from_str_radix(id, 16) {
                        Ok(v) => v,
                        Err(_) => {
                            number = false;
                            0
                        }
                    };
                    let mut found = false;
                    let mut item_id = 0;
                    if number {
                        item_id = match name_map.get(&num) {
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
                            if name == id.trim() || name.to_lowercase() == id.trim().to_lowercase() {
                                item_id = *n;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        println!("No such item: {}", id);
                        continue;
                    } else {
                        ids.push(item_id);
                    }
                }
                let mut amount: u32;
                loop {
                    let stop;
                    let line = read_input("Enter an amount: ");

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
                let mut start_slot: u32;
                loop {
                    let stop;
                    let line = read_input("Enter a starting slot: ");

                    start_slot = match line.parse() {
                        Ok(num) => {
                            if (num + ids.len() as u32) > 35 {
                                stop = false;
                                println!("The number of items exceeds the maximum amount of slots.");
                                0
                            } else {
                                stop = true;
                                num
                            }
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
                let mut index = 0;
                for id in ids {
                    write_slot((start_slot+index) as usize, id, amount, &mut buf);
                    let mut save_file = File::create(filename).unwrap();
                    index+=1;
                    save_file.write_all(&buf).unwrap();
                }
            }
            "exit" => {
                std::process::exit(1);
            }
            _ => {
                println!("No such command: {}", line)
            }
        }
    }
}
