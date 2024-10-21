mod cut;
use std::{
    env,
    io::{BufRead, BufReader},
};

fn main() {
    let cmd_argument: Vec<String> = env::args().collect();
    let config = cut::CutConfig::parse(cmd_argument);
    let mut output: Vec<String> = Vec::new();

    // TODO: Not great to expose this but it is a toy project
    if let Some(file) = &config.input_file {
        let buf_reader = BufReader::new(file);
        for line in buf_reader.lines() {
            config.process(&line.unwrap(), &mut output);
        }
    };

    if let Some(stdin) = &config.stdin {
        loop {
            let mut buf_reader: BufReader<_> = BufReader::new(stdin);
            let mut line = String::new();
            let written = buf_reader.read_line(&mut line).unwrap_or(0);
            if written == 0 {
                break;
            }
            config.process(&line, &mut output);
        }
        return;
    }

    for o in output.into_iter() {
        println!("{}", o);
    }
    return;
}
