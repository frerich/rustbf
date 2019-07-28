use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;


enum ParseError {
}


enum Command {
    IncDataPtr,
    DecDataPtr,
    Inc,
    Dec,
    Read,
    Write,
    Loop(Program),
}


struct Program {
    instructions: Vec<Command>
}


struct Machine {
    memory: Vec<u8>,
    data_ptr: usize
}


impl Program {
    fn new() -> Program {
        Program { instructions: Vec::new() }
    }

    fn parse(code: &str) -> Result<Program, ParseError> {
        let mut programs = Vec::new();
        programs.push(Program::new());

        for c in code.chars() {
            let p = programs.last_mut().unwrap();
            match c {
                '>' => p.instructions.push(Command::IncDataPtr),
                '<' => p.instructions.push(Command::DecDataPtr),
                '+' => p.instructions.push(Command::Inc),
                '-' => p.instructions.push(Command::Dec),
                '.' => p.instructions.push(Command::Write),
                ',' => p.instructions.push(Command::Read),
                '[' => programs.push(Program::new()),
                ']' => {
                    let loop_body = programs.pop().unwrap();
                    programs.last_mut().unwrap().instructions.push(Command::Loop(loop_body));
                }
                _  => {}
            };
        }

        Ok( programs.pop().unwrap() )
    }
}


impl Machine {
    fn new(mem_size: usize) -> Machine {
        Machine {
            memory: vec![0; mem_size],
            data_ptr: 0
        }
    }

    fn run(self: &mut Machine, program: &Program) -> io::Result<()> {
        for inst in &program.instructions {
            match inst {
                Command::IncDataPtr => self.data_ptr += 1,
                Command::DecDataPtr => self.data_ptr -= 1,
                Command::Inc => self.memory[self.data_ptr] += 1,
                Command::Dec => self.memory[self.data_ptr] -= 1,
                Command::Write => {
                    io::stdout().write(&[self.memory[self.data_ptr]])?;
                    io::stdout().flush()?;
                },
                Command::Read => {
                    let mut buf = vec![0];
                    io::stdin().read(&mut buf)?;
                    self.memory[self.data_ptr] = buf[0];
                },
                Command::Loop(body) => {
                    while self.memory[self.data_ptr] != 0 {
                        self.run(&body)?;
                    }
                },
            }
        }
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print!("usage: rustbf <filename>");
        return;
    }

    let file_name = &args[1];

    let code: String = match fs::read_to_string(file_name) {
        Err(e) => { print!("{}", e); return; },
        Ok(s) => s
    };

    match Program::parse(&code) {
        Err(_) => return,
        Ok(program) => {
            let mut machine = Machine::new(65536);
            machine.run(&program);
        }
    }
}
