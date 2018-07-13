extern crate rust_tc;

use std::env;
use std::str::from_utf8;

use rust_tc::{BDB, BDBOWRITER, BDBOCREAT, Result};

pub fn sometest() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("not enough args");
        return Ok(())
    }
    let cmd = &args[1];

    let mut db = BDB::new();

    db.open("test.db", BDBOWRITER|BDBOCREAT)?;

    match cmd.as_ref() {
        "get" => {
            if args.len() < 3 {
                println!("not enough args");
                return Ok(())
            }
            if let Some(v) = db.get(args[2].as_ref()) {
                println!("{}", std::str::from_utf8(&v).unwrap());
            }
        },
        "put" => {
            if args.len() < 4 {
                println!("not enough args");
                return Ok(())
            }
            db.put(args[2].as_ref(), args[3].as_ref())?;
        },
        "range" => {
            if args.len() < 4 {
                println!("not enough args");
                return Ok(())
            }
            for v in &db.range(args[2].as_ref(), true, args[3].as_ref(), true, -1) {
                println!("{}", from_utf8(v).unwrap());
            }
        },
        "prefix" => {
            if args.len() < 3 {
                println!("not enough args");
                return Ok(())
            }
            for v in &db.fwmkeys(args[2].as_ref(), -1) {
                println!("{}", from_utf8(v).unwrap());
            }
        },
        "list" => {
            let mut c = db.cursor();
            c.first();
            while let Some(key) = c.key() {
                if let Some(val) = c.val() {
                    println!("{} {}", from_utf8(&key).unwrap(), from_utf8(&val).unwrap())
                }
                c.next();
            }
        },
        "path" => {
            println!("{}", db.path().unwrap());
        },
        cmd => {
            println!("unknown command: {}", cmd);
        }
    }

    db.close()?;

    Ok(())
}


fn main() {
    if let Err(e) = sometest() {
        println!("Error: {:?}", e);
    }

    //rust_tc::cmptest();
    /*
    let mut l = List::new();

    l.push(b"tom");
    l.push(b"dick");
    l.push(b"harry");

    println!("len(l) = {}", l.len());

    for v in &l {
        println!("{}", std::str::from_utf8(v).unwrap());
    }
    */
}
