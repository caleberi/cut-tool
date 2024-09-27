use std::vec::IntoIter;
use std::{env, io};

struct CutConfig {
    //  -d delim Use delim as the field delimiter character instead of the tab character.
    delimiter: String,
    // -f list The list specifies fields, separated in the input by the field delimiter character
    //  (see the -d option).  Output fields are separated by a single occurrence of the
    //  field delimiter character.
    fields: Vec<u64>,
    //  If no file arguments are specified, or a file
    // argument is a single dash (‘-’), cut reads from the standard input.
    input: Option<Box<dyn io::Read>>,

    // Use whitespace (spaces and tabs) as the
    // delimiter.  Consecutive spaces and tabs count as
    // one single field separator.
    whitespace: bool,
    // Suppress lines with no field delimiter
    // characters.  Unless specified, lines with no
    // delimiters are passed through unmodified.
    suppress: bool,

    // The list specifies byte positions.
    byte_pos: Vec<u8>,

    // The list specifies character positions.
    char_pos: Vec<u64>,

    // Do not split multibyte characters.  Characters
    // will only be output if at least one byte is
    // selected, and, after a prefix of zero or more
    // unselected bytes, the rest of the bytes that form
    // the character are selected.
    no_split: bool,
}

impl CutConfig {
    fn new() -> CutConfig {
        CutConfig {
            delimiter: "".to_string(),
            fields: vec![],
            input: None,
            whitespace: false,
            suppress: false,
            byte_pos: vec![],
            char_pos: vec![],
            no_split: false,
        }
    }

    fn is_digit(s: &str) -> bool {
        s.parse::<i32>().is_ok() && !s.parse::<f32>().is_err() // rejecting float
    }

    fn process_field_token(self: &mut Self, string_fields: &str) {
        let is_range = string_fields.contains("-");
        if is_range {
            let val: Vec<&str> = string_fields
                .split(",")
                .collect();
            let mut v = Vec::<u64>::new();
            self.handle_range_fields(val, &mut v);
            return;
        }
        self.fields = string_fields
            .split(",")
            .map(|v| v.trim())
            .filter_map(|v| v.parse::<u64>().ok())
            .filter(|v| *v != 0)
            .collect();
    }

    fn handle_range_fields(self: &mut Self, val: Vec<&str>, v: &mut Vec<u64>) {
        for n in 0..val.len() {
            let is_range = val.get(n).unwrap().contains("-");
            if is_range {
                let imd: Vec<&str> = val.get(n).unwrap().split("-").collect();
                let x = imd[0].parse::<u64>().unwrap();
                let y = imd[1].parse::<u64>().unwrap();
                for a in x..=y {
                    v.push(a);
                }
                continue;
            }
            let x = val.get(n).unwrap().parse::<u64>().unwrap();
            v.push(x);
        }
        v.sort();
        self.fields = v.clone();
    }
}

fn main() {
    // TODO:  How to handle argument in rust
    let cmd_argument: Vec<String> = env::args().collect();
    let mut cmd_itr = cmd_argument.into_iter();
    _ = cmd_itr.next();
    let _config = parse_commandline_arg(cmd_itr).unwrap();

    println!("_config.fields : {:?}", _config.fields);
    println!("_config.delimiter : {:?}", _config.delimiter);
    println!("_config.byte_pos : {:?}", _config.byte_pos);
    println!("_config.whitespace : {:?}", _config.whitespace);
    println!("_config.char_pos : {:?}", _config.char_pos);
    println!("_config.no_split : {:?}", _config.no_split);

    // TODO  Handle how we can get file content from stdin before applying
    // logic to the  content

    //TODO: Figure out how to configure flags

}

fn parse_commandline_arg(mut itr: IntoIter<String>) -> Result<CutConfig, String> {
    let mut config = CutConfig::new();
    while let Some(token) = itr.next() {
        let str_token = token.as_str();
        let prefix_flag = str_token
            .get(0..2)
            .unwrap_or_else(|| return str_token.get(0..1).unwrap_or(""));

        if !prefix_flag.starts_with("-") {
            let file = std::fs::File::open(&str_token);
            if let Err(err) = file {
                return Err(err.to_string());
            }
            config.input = Some(Box::new(file.unwrap()));
        } else {
            match prefix_flag {
                "-" => config.input = Some(Box::new(io::stdin())),
                "-f" => {
                    let string_fields = str_token.strip_prefix("-f").unwrap();
                    config.process_field_token(string_fields);
                }
                "-d" => {
                    let delimiter = str_token.strip_prefix("-d").unwrap_or("").to_string();
                    config.delimiter = delimiter;
                }
                "-n" => config.no_split = true,
                "-w" => config.whitespace = true,
                "-s" => config.suppress = true,
                "-b" => {
                    let string_fields = str_token.strip_prefix("-b").unwrap();
                    config.byte_pos = string_fields
                        .split(",")
                        .filter(|x| CutConfig::is_digit(*x))
                        .filter_map(|v| v.parse::<u8>().ok())
                        .collect();
                }
                "-c" => {
                    config.char_pos = str_token
                        .strip_prefix("-c")
                        .unwrap()
                        .split(",")
                        .filter(|x| CutConfig::is_digit(*x))
                        .filter_map(|v| v.parse::<u64>().ok())
                        .collect();
                }
                filename => {
                    // check if the file name is blank
                    // try to access the file if it exists
                    //  not exist - error
                    // else set the input configs
                }
            }
        }
    }

    Ok(config)
}
