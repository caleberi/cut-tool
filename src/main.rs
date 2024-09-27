use std::{env, io};
use std::vec::IntoIter;

struct CutConfig {
    //  -d delim Use delim as the field delimiter character instead of the tab character.
    delimiter: String,
    // -f list The list specifies fields, separated in the input by the field delimiter character
    //  (see the -d option).  Output fields are separated by a single occurrence of the
    //  field delimiter character.
    field: Vec<i32>,
    //  If no file arguments are specified, or a file
    // argument is a single dash (‘-’), cut reads from the standard input.
    input: Option<Box::<dyn io::Read>>,
}



impl  CutConfig {
    fn new() -> CutConfig {
        CutConfig{ delimiter: "".to_string(), field: vec![], input: None,}
    }

    fn process_field_token(self: &mut Self, string_fields: &str) {
        fn is_digit(s : &str) -> bool {
            s.as_bytes() >= b"0"  && s.as_bytes() <= b"9"
        }

        let is_range = string_fields.contains("-");
        if is_range {
            let val: Vec<&str> = string_fields.split(",")
                .filter(|x|is_digit(*x))
                .collect();
            let mut v = Vec::<i32>::new();
            self.handle_range_fields( val, &mut v);
            return;
        }
        self.field =  string_fields.split(",")
            .map(|v| v.trim())
            .filter_map(|v| v.parse::<i32>().ok())
            .filter(|v| *v != 0)
            .collect();
    }

    fn handle_range_fields(self: &mut Self, val: Vec<&str>, v: &mut Vec<i32>) {
        for n in 0..val.len() {
            let is_range = val.get(n).unwrap().contains("-");
            if is_range {
                let imd: Vec<&str> = val.get(n).unwrap().split("-").collect();
                let x = imd[0].parse::<i32>().unwrap();
                let y = imd[1].parse::<i32>().unwrap();
                for a in x..=y { v.push(a); }
                continue
            }
            let x = val.get(n).unwrap().parse::<i32>().unwrap();
            v.push(x);
        }
        v.sort();
        self.field = v.clone();
    }
}

fn main() {
    // TODO:  How to handle argument in rust
    let cmd_argument : Vec<String> = env::args().collect();
    let mut  cmd_itr  = cmd_argument.into_iter();
    _ = cmd_itr.next();
    let _config = parse_commandline_arg(cmd_itr).unwrap();

    println!("_config.delimiter : {:?}",_config.field);
    println!("_config.delimiter : {:?}",_config.delimiter);

   //TODO: Figure out how to configure flags
    // TODO:  Figure out if we were given a file or reading stdin
}

fn parse_commandline_arg(mut itr:  IntoIter<String>) -> Result<CutConfig,String> {
    let mut config =  CutConfig::new();
    while let Some(token) = itr.next() {
        let str_token = token.as_str();
        let prefix_flag = str_token.get(0..2).unwrap_or_else(|| {
            return str_token.get(0..1).unwrap_or("")
        });

        if !prefix_flag.starts_with("-") {
            let file = std::fs::File::open(&str_token);
            if let Err(err) = file {
                return Err(err.to_string())
            }
            config.input = Some(Box::new(file.unwrap()));
        } else {
            match prefix_flag {
                "-" => config.input = Some(Box::new(io::stdin())),
                "-f"=> {
                    let string_fields = str_token.strip_prefix("-f").unwrap();
                    config.process_field_token(string_fields);
                },
                "-d"=>{
                    let delimiter= str_token.strip_prefix("-d")
                        .unwrap_or("").to_string();
                    config.delimiter = delimiter;
                },
                _ => {}
            }
        }
    }

    Ok(config)
}

