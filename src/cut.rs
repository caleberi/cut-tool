pub mod cut_tool {
    use std::collections::HashMap;
    use std::str::Chars;
    use std::vec::IntoIter;
    use std::{fs, io};
    use unicode_segmentation::UnicodeSegmentation;

    pub struct CutConfig {
        //  -d delim Use delim as the field delimiter character instead of the tab character.
        delimiter: String,
        // -f list The list specifies fields, separated in the input by the field delimiter character
        //  (see the -d option).  Output fields are separated by a single occurrence of the
        //  field delimiter character.
        fields: Vec<u64>,
        //  If no file arguments are specified, or a file
        // argument is a single dash (‘-’), cut reads from the standard input.
        pub input_file: Option<fs::File>,

        pub stdin: Option<io::Stdin>,

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
        pub fn parse(args: Vec<String>) -> CutConfig {
            let mut cmd_itr = args.into_iter();
            _ = cmd_itr.next();

            let msg = " 
            SYNOPSIS
                    cut -b list [-n] [file ...]
                    cut -c list [file ...]
                    cut -f list [-w | -d delim] [-s] [file ...]
            ";
            let config = if let Ok(cfg) = parse_commandline_arg(cmd_itr) {
                if cfg.help {
                    println!("{msg}");
                    std::process::exit(1);
                }
                cfg
            } else {
                println!("{msg}");
                std::process::exit(1);
            };

            config
        }

        pub fn new() -> CutConfig {
            CutConfig {
                delimiter: String::from("\t"),
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
            let f: Vec<&str> = "1234567890.".split("").collect();
            let hay: Vec<&str> = s.split("").collect();
            for c in hay {
                if !&f.contains(&c) {
                    return false;
                }
            }
            fn wrap(s: &str) -> Result<bool, bool> {
                let x = s.parse::<i32>().is_ok();
                let y = s.parse::<f32>().is_ok();
                Ok(x || y)
            }
            return wrap(s).is_ok();
        }

        fn process_field_token(self: &mut Self, string_fields: &str) {
            let mut t_string_fields = String::from(string_fields);
            let has_space = string_fields.contains(" ");
            if has_space {
                t_string_fields = t_string_fields.replace(" ", ",");
            }
            let is_range = t_string_fields.contains("-");
            if is_range {
                let val: Vec<&str> = t_string_fields
                    .split(",")
                    .filter(|v| v.len() != 0)
                    .collect();
                let mut v = Vec::<u64>::new();
                self.handle_range_fields(val, &mut v);
                return;
            }
            self.fields = t_string_fields
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

        pub fn process(self: &Self, line: &String, output: &mut Vec<String>) {
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

            let mut byte_positions = Vec::new();
            if self.no_split {
                for grapheme in graphemes.iter() {
                    let char_end = char_index + grapheme.len();
                    for n in &self.byte_pos {
                        let n = *n as usize;
                        if char_index <= n && n < char_end {
                            byte_positions.push(grapheme.to_string());
                        }
                    }

                    char_index = char_end;
                }
                output.push(byte_positions.join(" "));
                return;
            }

            let bytes = line.as_bytes();
            for n in &self.byte_pos {
                let v = bytes[*n as usize];
                byte_positions.push(String::from(char::from(v)));
            }
            output.push(byte_positions.join(" "));
        }

        fn handle_character_query(self: &Self, line: &String, output: &mut Vec<String>) {
            let characters: Chars<'_> = line.chars().into_iter();
            let mut indexes = HashMap::new();
            for (idx, character) in characters.enumerate() {
                _ = indexes.insert(idx, character);
            }
            let mut values: Vec<String> = vec![];
            for c in self.char_pos.iter() {
                let key = *c as usize;
                if let Some(v) = indexes.get(&key) {
                    values.push(v.to_string());
                }
            }
            output.push(values.join("\t"));
        }

        fn handle_file_query(self: &Self, line: &String, output: &mut Vec<String>) {
            if self.suppress && !line.contains(&self.delimiter) {
                return;
            }

            let tokens: Vec<&str> = line.split(&self.delimiter).collect();

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
                config.input_file = Some(file.unwrap());
            } else {
                match prefix_flag {
                    "-h" => config.help = true,
                    "-" => config.stdin = Some(io::stdin()),
                    "-f" => {
                        let string_fields = str_token.strip_prefix("-f").unwrap();
                        config.process_field_token(string_fields);
                    }
                    "-d" => {
                        let delimiter = str_token.strip_prefix("-d").unwrap_or("\t").to_string();
                        if config.whitespace && config.delimiter.as_str() != "\t" {
                            config.delimiter = String::from("\t");
                        } else {
                            config.delimiter = delimiter;
                        }
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

    #[cfg(test)]
    mod test {
        use std::io::{BufRead, BufReader};

        use crate::cut::cut_tool::CutConfig;
        struct Suit {
            input: &'static str,
            expected: bool,
        }

        fn init_config(args: &str) -> CutConfig {
            let arg = String::from(args);
            let cmd_argument: Vec<String> = arg
                .split(" ")
                .map(|x| x.to_string())
                .filter(|x| x.len() != 0)
                .collect();
            CutConfig::parse(cmd_argument)
        }

        #[test]
        fn test_is_digit() {
            let mut tests: Vec<Suit> = Vec::new();
            tests.push(Suit {
                input: "34",
                expected: true,
            });
            tests.push(Suit {
                input: "3.34",
                expected: true,
            });
            tests.push(Suit {
                input: "14.9",
                expected: true,
            });
            tests.push(Suit {
                input: "3a4",
                expected: false,
            });

            tests.push(Suit {
                input: "3*4",
                expected: false,
            });
            for test in tests {
                assert_eq!(CutConfig::is_digit(test.input), test.expected)
            }
        }

        #[test]
        fn test_process() {
            let config = init_config("cut-tool -f1,2,3,4,5  test/sample.tsv");
            let mut output: Vec<String> = Vec::new();
            if let Some(file) = &config.input_file {
                let buf_reader = BufReader::new(file);
                for line in buf_reader.lines() {
                    let v = &line.unwrap();
                    config.process(v, &mut output);
                }
            };
            let result =
             "f0\tf1\tf2\tf3\tf4\n0\t1\t2\t3\t4\n5\t6\t7\t8\t9\n10\t11\t12\t13\t14\n15\t16\t17\t18\t19\n20\t21\t22\t23\t24\n";

            let mut buffer = String::new();
            for o in output.into_iter() {
                buffer.push_str(o.as_str());
                buffer.push('\n');
            }
            assert_eq!(result, buffer)
        }

        #[test]
        fn test_process_field_token() {
            let config = init_config("cut-tool -f1,2,3,4,5  test/sample.tsv");
            let mut expected: Vec<u64> = Vec::new();
            {
                expected.push(1);
                expected.push(2);
                expected.push(3);
                expected.push(4);
                expected.push(5);
            }
            assert_eq!(config.fields, expected);
        }

        #[test]
        fn test_handle_range_fields() {
            let config = init_config("cut-tool -f1-4,5  test/sample.tsv");
            let mut expected: Vec<u64> = Vec::new();
            {
                expected.push(1);
                expected.push(2);
                expected.push(3);
                expected.push(4);
                expected.push(5);
            }
            assert_eq!(config.fields, expected);
        }
    }
}
