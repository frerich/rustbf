use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;


enum ParseError {
}


enum BFCommand {
    IncDataPtr,
    DecDataPtr,
    Inc,
    Dec,
    Read,
    Write,
    Loop(BFProgram),
}


struct BFProgram {
    instructions: Vec<BFCommand>
}


struct Machine {
    memory: Vec<u8>,
    data_ptr: usize
}


impl BFProgram {
    fn new() -> BFProgram {
        BFProgram { instructions: Vec::new() }
    }

    fn parse(code: &str) -> Result<BFProgram, ParseError> {
        let mut programs = Vec::new();
        programs.push(BFProgram::new());

        for c in code.chars() {
            let p = programs.last_mut().unwrap();
            match c {
                '>' => p.instructions.push(BFCommand::IncDataPtr),
                '<' => p.instructions.push(BFCommand::DecDataPtr),
                '+' => p.instructions.push(BFCommand::Inc),
                '-' => p.instructions.push(BFCommand::Dec),
                '.' => p.instructions.push(BFCommand::Write),
                ',' => p.instructions.push(BFCommand::Read),
                '[' => programs.push(BFProgram::new()),
                ']' => {
                    let loop_body = programs.pop().unwrap();
                    programs.last_mut().unwrap().instructions.push(BFCommand::Loop(loop_body));
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

    fn run(self: &mut Machine, program: &BFProgram) -> io::Result<()> {
        for inst in &program.instructions {
            match inst {
                BFCommand::IncDataPtr => self.data_ptr += 1,
                BFCommand::DecDataPtr => self.data_ptr -= 1,
                BFCommand::Inc => self.memory[self.data_ptr] += 1,
                BFCommand::Dec => self.memory[self.data_ptr] -= 1,
                BFCommand::Write => {
                    io::stdout().write(&[self.memory[self.data_ptr]])?;
                },
                BFCommand::Read => {
                    let mut buf = vec![0];
                    io::stdin().read(&mut buf)?;
                    self.memory[self.data_ptr] = buf[0];
                },
                BFCommand::Loop(body) => {
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

    match BFProgram::parse(&code) {
        Err(_) => return,
        Ok(program) => {
            let mut machine = Machine::new(65536);
            machine.run(&program);
        }
    }
}
