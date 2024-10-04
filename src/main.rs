use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::str::Chars;
use std::vec::IntoIter;
use std::{env, fs, io};
use unicode_segmentation::UnicodeSegmentation;

struct CutConfig {
    //  -d delim Use delim as the field delimiter character instead of the tab character.
    delimiter: String,
    // -f list The list specifies fields, separated in the input by the field delimiter character
    //  (see the -d option).  Output fields are separated by a single occurrence of the
    //  field delimiter character.
    fields: Vec<u64>,
    //  If no file arguments are specified, or a file
    // argument is a single dash (‘-’), cut reads from the standard input.
    input_file: Option<Box<fs::File>>,

    stdin: Option<Box<io::Stdin>>,

    // Use whitespace (spaces and tabs) as the
    // delimiter.  Consecutive spaces and tabs count as
    // one single field separator.
    whitespace: bool,
    // Suppress lines with no field delimiter
    // characters.  Unless specified, lines with no
    // delimiters are passed through unmodified.
    suppress: bool,

    // The list specifies byte positions.
    byte_pos: Vec<u64>,

    // The list specifies character positions.
    char_pos: Vec<u64>,

    // Do not split multibyte characters.  Characters
    // will only be output if at least one byte is
    // selected, and, after a prefix of zero or more
    // unselected bytes, the rest of the bytes that form
    // the character are selected.
    no_split: bool,
    help: bool,
}

impl CutConfig {
    fn new() -> CutConfig {
        CutConfig {
            delimiter: " ".to_string(),
            fields: vec![],
            input_file: None,
            stdin: None,
            whitespace: false,
            suppress: false,
            byte_pos: vec![],
            char_pos: vec![],
            help: false,
            no_split: false,
        }
    }

    fn is_digit(s: &str) -> bool {
        s.parse::<i32>().is_ok() && !s.parse::<f32>().is_err() // rejecting float
    }

    fn process_field_token(self: &mut Self, string_fields: &str) {
        let is_range = string_fields.contains("-");
        if is_range {
            let val: Vec<&str> = string_fields.split(",").collect();
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

    fn process(self: &Self, line: &String, output: &mut Vec<String>) {
        if self.byte_pos.len() > 0 {
            self.handle_byte_query(line, output);
            return;
        }

        if self.char_pos.len() > 0 {
            self.handle_character_query(line, output);
            return;
        }

        self.handle_file_query(line, output);
    }

    fn handle_byte_query(self: &Self, line: &String, output: &mut Vec<String>) {
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let mut char_index = 0;

        if self.no_split {
            for n in &self.byte_pos {
                for grapheme in graphemes.clone().into_iter() {
                    let char_end = char_index + grapheme.len();

                    if char_end > (*n as usize) && (*n as usize) < char_index {
                        output.push(grapheme.to_string());
                    }

                    char_index = char_end;
                }
            }
        }
        for n in &self.byte_pos {
            let v = graphemes.get(*n as usize).unwrap();
            output.push(v.to_string());
        }
    }

    fn handle_character_query(self: &Self, line: &String, output: &mut Vec<String>) {
        let characters: Chars<'_> = line.chars().into_iter();
        let mut indexes = HashMap::new();
        for (idx, character) in characters.enumerate() {
            _ = indexes.insert(idx, character).unwrap();
        }
        for c in self.char_pos.iter() {
            let key = *c as usize;
            if let Some(v) = indexes.get(&key) {
                output.push(v.to_string());
            }
        }
    }

    fn handle_file_query(self: &Self, line: &String, output: &mut Vec<String>) {
        if self.suppress && !line.contains(&self.delimiter) {
            return;
        }
        let mut dem_tab_or_space = false;
        if !self.whitespace && self.delimiter == " " {
            dem_tab_or_space = true;

            let tokens: Vec<&str> = line.split("\t").collect();

            let mut values: Vec<&str> = vec![];
            for val in &self.fields {
                let value = *val - 1;

                if value < tokens.len() as u64 {
                    let token = tokens[value as usize];
                    values.push(token);
                } else {
                    values.push(" ");
                }

            }
            output.push(values.join("\t"));
        }

        _ = dem_tab_or_space;
    }
}

fn main() {
    let cmd_argument: Vec<String> = env::args().collect();
    let mut cmd_itr = cmd_argument.into_iter();
    _ = cmd_itr.next();

    let config = parse_commandline_arg(cmd_itr).unwrap();
    let mut output: Vec<String> = Vec::new();
    if config.help {
        let msg = " 
        SYNOPSIS
             cut -b list [-n] [file ...]
             cut -c list [file ...]
             cut -f list [-w | -d delim] [-s] [file ...]
        ";
        println!("{msg}");
        return;
    }
    if config.input_file.is_some() {
        let file = &config.input_file.as_ref().unwrap();
        let buf_reader = BufReader::new(file.as_ref());

        for line in buf_reader.lines() {
                // match &line {
                //     Ok(size) => { if size == 0 {break;} }
                //     Err(e) => panic!("{}", e.to_string().as_str()),
                // }
            config.process(&line.unwrap(), &mut output);
        }

        for o in output.into_iter() {
            println!("{}", o);
        }
        return;
    }

    if config.stdin.is_some() {
        return;
    }
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
            config.input_file = Some(Box::new(file.unwrap()));
        } else {
            match prefix_flag {
                "-h" => config.help = true,
                "-" => config.stdin = Some(Box::new(io::stdin())),
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
                        .filter_map(|v| v.parse::<u64>().ok())
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
                _ => (),
            }
        }
    }

    Ok(config)
}
