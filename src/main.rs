extern crate byteorder;

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, stdout, Write, Cursor};
use byteorder::{BigEndian, ReadBytesExt, LittleEndian};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please select a save file");
        std::process::exit(1);
    }
    let filename = &args[1];
    let mut save_file = File::open(filename).unwrap();
    let mut buf = vec![];
    save_file.read_to_end(&mut buf).unwrap();
    //println!("{:x}", buf[0x1dc + 12]);

    let mut stdin = std::io::stdin();

    loop {
        print!("> ");
        stdout().flush();
        let line = stdin
            .lock()
            .lines()
            .next()
            .expect("there was no next line")
            .expect("the line could not be read");
        match line.as_str() {
            "help" => {
                //TODO add help
            },
            "list" => {
                let mut offset = 0x1dc;
                loop {
                    //Inventory ends at 0x383
                    if offset > 0x383 {
                        break;
                    }
                    //let id = ((((buf[offset + 1] as u16) & 0xFF) << 8) | ((buf[offset] as u16) & 0xFF));
                    let mut idv = Cursor::new(vec![buf[offset + 1], buf[offset]]);
                    let id = idv.read_u16::<BigEndian>().unwrap();
                    let mut amount_slice = Cursor::new(&buf[offset + 2..offset + 10]);
                    let amount = amount_slice.read_u64::<LittleEndian>().unwrap();
                    println!("Slot: {} ID: 0x{:x}, Amount: {}", (offset - 0x1dc) / 12, id, amount);
                    offset += 12;
                }
            }
            _ => {
                println!("No such command: {}", line)
            }
        }
    }
}
