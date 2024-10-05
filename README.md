# Cut Tool

This is a Rust implementation of the Unix `cut` command, which extracts sections from each line of input.

## Features

- Supports cutting by fields, bytes, or characters
- Allows custom field delimiters
- Can handle input from files or standard input
- Supports various options similar to the Unix `cut` command

## Usage

```
cut -b list [-n] [file ...]
cut -c list [file ...]
cut -f list [-w | -d delim] [-s] [file ...]
```

### Options

- `-b list`: The list specifies byte positions to extract
- `-c list`: The list specifies character positions to extract
- `-f list`: The list specifies fields to extract
- `-d delim`: Use `delim` as the field delimiter character instead of the tab character
- `-w`: Use whitespace (spaces and tabs) as the delimiter
- `-s`: Suppress lines with no field delimiter characters
- `-n`: Do not split multibyte characters (only applicable with `-b`)
- `-h`: Display help information

### Input

If no file arguments are specified, or if a file argument is a single dash ('-'), the tool reads from standard input.

## Examples

1. Cut by fields:
   ```
   ./cut -f 1,3 -d ',' input.csv
   ```
   This extracts the 1st and 3rd fields from each line in `input.csv`, using comma as the delimiter.

2. Cut by bytes:
   ```
   ./cut -b 1-5,10-15 input.txt
   ```
   This extracts bytes 1 to 5 and 10 to 15 from each line in `input.txt`.

3. Cut by characters:
   ```
   ./cut -c 1-10 input.txt
   ```
   This extracts the first 10 characters from each line in `input.txt`.

4. Use with standard input:
   ```
   echo "Hello,World,Test" | ./cut -f 2 -d ','
   ```
   This will output "World".

## Building

To build the project, make sure you have Rust installed and run:

```
cargo build --release
```

The executable will be available in the `target/release` directory.

## Notes

- The tool aims to be compatible with the Unix `cut` command, but there might be some differences in behavior.
- When using the `-b` option with `-n`, the tool ensures that multibyte characters are not split.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.