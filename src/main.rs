use std::env;
use std::fs::File;
use std::io::Read;
use std::io::stdin;
use std::path::Path;
use std::process::exit;
use log::{Level, log};
use std::str;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
            eprint!("You have to supply pathname to .bf file");
            exit(1);
    }
    let filepath = &args[1];

    let mut interpreter: InterpreterState = Interpreter::new();
    interpreter.load_file(filepath);
    interpreter.parse(None);
}

#[derive(Debug)]
struct InterpreterState {
    cells: [u8; 30000],
    cell_index: usize,
    loops_opened: usize,
    file_content: String,
    loops_data: Vec<[usize; 2]>,
}


trait Interpreter {
    fn new() -> Self;
    fn load_file(&mut self, filename: &str);
    fn parse(&mut self, items: Option<&String>);
    fn execute_loop_context(&mut self);

    // Language operations
    fn increment(&mut self);
    fn decrement(&mut self);
    fn goto_next_cell(&mut self);
    fn goto_previous_cell(&mut self);
    fn open_loop(&mut self, current_parser_index: usize);
    fn close_loop(&mut self, current_parser_index: usize);
    fn print(&self);
    fn input(&mut self);
}


impl Interpreter for InterpreterState {
    fn new() -> Self {
        return Self {
            cell_index: 0,
            cells: [0; 30000],
            file_content: String::new(),
            loops_opened: 0,
            loops_data: Vec::new(),
        };
    }

    fn load_file(&mut self, input_filename: &str) {
        let path = Path::new(input_filename);
        if !path.exists() {
            panic!("File {path:?} does not exist");
        }
        let mut file = match File::open(path) {
            Err(_) => panic!("could not read file"),
            Ok(content) => content,
        };

        file.read_to_string(&mut self.file_content)
            .expect("Could not read to buffer");
    }

    fn parse(&mut self, items: Option<&String>) {
        let chars: Vec<char> = match items {
            Some(val) => val.chars().collect(),
            None => self.file_content.chars().collect(),
        };

        for (index, char) in chars.iter().enumerate() {
            match char {
                '+' => self.increment(),
                '-' => self.decrement(),
                '>' => self.goto_next_cell(),
                '<' => self.goto_previous_cell(),
                '.' => self.print(),
                '[' => {
                    self.open_loop(index);
                },
                ']' => {
                    self.close_loop(index);
                },
                ',' => self.input(),
                _ => log!(Level::Debug, "Passed other char, treating as comment"),
            }
        }
    }

    fn execute_loop_context(&mut self) {
        let loop_data = self.loops_data[self.loops_opened - 1];
        if loop_data[1] == 0 {
            panic!("Missing enclosing ']' near char nr")
        }
        while self.cells[self.cell_index] > 0 {
            let slice = &self.file_content[
                loop_data[0]..loop_data[1]
                ].to_string();
            self.parse(Some(slice))
        }

        self.loops_data.clear();
    }


    fn increment(&mut self) {
        if self.cells[self.cell_index] == 255 {
            self.cells[self.cell_index] = 0;
        } else {
            self.cells[self.cell_index] = 255;
        }

    }

    fn decrement(&mut self) {
        if self.cells[self.cell_index] == 0 {
            self.cells[self.cell_index] = 255;
        } else {
            self.cells[self.cell_index] -= 1;
        }
    }

    fn goto_next_cell(&mut self) {
        if self.cell_index == 29999 {
            self.cell_index = 0;
        } else {
            self.cell_index += 1;
        }
    }

    fn goto_previous_cell(&mut self) {
        if self.cell_index == 0 {
            self.cell_index = 29999;
        } else {
            self.cell_index -= 1;
        }
    }

    fn open_loop(&mut self, current_parser_index: usize) {
        self.loops_data.push([current_parser_index, 0]);
        self.loops_opened += 1;
        log!(Level::Debug, "End of the loop as of now")
    }

    fn close_loop(&mut self, current_parser_index: usize) {
        if self.loops_opened == 0 {
            eprintln!("Syntax Error: Trying to close loop, but there's no opened loop.")
        } else {
            let loops_cnt = self.loops_data.len();

            self.loops_data[loops_cnt - 1][1] = current_parser_index;

            if self.loops_opened == loops_cnt {
                self.execute_loop_context();
            }

            self.loops_opened -= 1;
        }
    }

    fn print(&self) {
        match str::from_utf8(&[self.cells[self.cell_index]]) {
            Ok(value) => print!("{value}"),
            Err(_) => println!("Invalid utf8 char"),
        }
    }

    fn input(&mut self) {
        let mut input = [0, 1];
        stdin().read(&mut input).unwrap();

        self.cells[self.cell_index] = input[0];
    }
}

#[cfg(test)]
mod tests {
    use crate::{Interpreter, InterpreterState};

    #[test]
    fn increment() {
        let mut i = InterpreterState::new();
        i.increment();

        assert_eq!(i.cells[0], 1);
    }

    #[test]
    fn decrement() {
        let mut i = InterpreterState::new();
        i.increment();
        i.increment();

        i.decrement();
        assert_eq!(i.cells[0], 1);
    }

    #[test]
    fn goto_next_call_at_end() {
        let mut i = InterpreterState::new();
        i.cell_index = 29999;

        i.goto_next_cell();

        assert_eq!(i.cell_index, 0);
    }

    #[test]
    fn goto_next_call_at_beginning() {
        let mut i = InterpreterState::new();
        i.goto_next_cell();

        assert_eq!(i.cell_index, 1);
    }

    #[test]
    fn goto_previous_cell_at_beginning() {
        let mut i = InterpreterState::new();
        i.goto_previous_cell();

        assert_eq!(i.cell_index, 29999);
    }

    #[test]
    fn goto_previous_cell_at_end() {
        let mut i = InterpreterState::new();
        i.goto_previous_cell();

        assert_eq!(i.cell_index, 29998);
    }
}
