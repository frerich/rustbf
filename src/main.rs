use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::collections::HashMap;


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


#[derive(Debug)]
enum BFPCommand {
    Update(MachineDelta),
    Multiply(HashMap<i32, i8>),
    Loop(BFPProgram),
    Read,
    Write
}


struct BFProgram {
    instructions: Vec<BFCommand>
}


#[derive(Debug)]
struct BFPProgram {
    instructions: Vec<BFPCommand>
}


#[derive(Debug)]
struct MachineDelta {
    memory: HashMap<i32, i8>,
    data_ptr: isize
}


struct Machine {
    memory: Vec<u8>,
    data_ptr: usize
}


impl MachineDelta {
    fn new() -> MachineDelta {
        MachineDelta { memory: HashMap::new(), data_ptr: 0 }
    }

    fn is_empty(self: &MachineDelta) -> bool {
        self.memory.is_empty() && self.data_ptr == 0
    }
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


impl BFPProgram {
    fn new() -> BFPProgram {
        BFPProgram { instructions: Vec::new() }
    }

    fn compile(program: &BFProgram) -> BFPProgram {
        let mut result = BFPProgram::new();
        let mut current_delta = MachineDelta::new();
        for inst in &program.instructions {
            match inst {
                BFCommand::IncDataPtr => current_delta.data_ptr += 1,
                BFCommand::DecDataPtr => current_delta.data_ptr -= 1,
                BFCommand::Inc => *current_delta.memory.entry(current_delta.data_ptr as i32).or_insert(0) += 1,
                BFCommand::Dec => *current_delta.memory.entry(current_delta.data_ptr as i32).or_insert(0) -= 1,
                BFCommand::Write => {
                    if !current_delta.is_empty() {
                        result.instructions.push(BFPCommand::Update(current_delta));
                        current_delta = MachineDelta::new();
                    }
                    result.instructions.push(BFPCommand::Write);
                },
                BFCommand::Read => {
                    if !current_delta.is_empty() {
                        result.instructions.push(BFPCommand::Update(current_delta));
                        current_delta = MachineDelta::new();
                    }
                    result.instructions.push(BFPCommand::Read);
                },
                BFCommand::Loop(body) => {
                    if !current_delta.is_empty() {
                        result.instructions.push(BFPCommand::Update(current_delta));
                        current_delta = MachineDelta::new();
                    }

                    let optimized_body = BFPProgram::compile(body);
                    if let [BFPCommand::Update(MachineDelta { memory, data_ptr: 0 })] = optimized_body.instructions.as_slice() {
                        result.instructions.push(BFPCommand::Multiply(memory.into_iter().filter(|&(k, _)| *k != 0).map(|(k,v)| (*k,*v)).collect()));
                    } else {
                        result.instructions.push(BFPCommand::Loop(optimized_body));
                    }
                },
            }
        }
        if !current_delta.is_empty() {
            result.instructions.push(BFPCommand::Update(current_delta));
        }
        result
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

    fn run_bfp_update(self: &mut Machine, delta: &MachineDelta) -> () {
        if !delta.memory.is_empty() {
            for (idx, val) in delta.memory.iter() {
                let mut absolute_idx = self.data_ptr as i64;
                absolute_idx += *idx as i64;

                let mut abs_val = self.memory[absolute_idx as usize] as i32;
                abs_val += *val as i32;
                self.memory[absolute_idx as usize] = abs_val as u8;
            }
        }

        if delta.data_ptr != 0 {
            let mut data_ptr = self.data_ptr as isize;
            data_ptr += delta.data_ptr;
            self.data_ptr = data_ptr as usize;
        }
    }

    fn run_bfp_multiply(self: &mut Machine, memory: &HashMap<i32, i8>) -> () {
        let factor = self.memory[self.data_ptr] as i32;
        if factor > 0 {
            for (idx, val) in memory.iter() {
                let mut absolute_idx = self.data_ptr as i64;
                absolute_idx += *idx as i64;

                let mut abs_val = self.memory[absolute_idx as usize] as i32;
                abs_val += *val as i32 * factor;
                self.memory[absolute_idx as usize] = abs_val as u8;
            }

            self.memory[self.data_ptr] = 0;
        }
    }

    fn run_bfp_loop(self: &mut Machine, body: &BFPProgram) -> io::Result<()> {
        while self.memory[self.data_ptr] != 0 {
            self.run_optimized(&body)?;
        }
        Ok(())
    }

    fn run_optimized(self: &mut Machine, program: &BFPProgram) -> io::Result<()> {
        for inst in &program.instructions {
            match inst {
                BFPCommand::Update(delta) => self.run_bfp_update(delta),
                BFPCommand::Multiply(memory) => self.run_bfp_multiply(memory),
                BFPCommand::Write => {
                    io::stdout().write(&[self.memory[self.data_ptr]])?;
                },
                BFPCommand::Read => {
                    let mut buf = vec![0];
                    io::stdin().read(&mut buf)?;
                    self.memory[self.data_ptr] = buf[0];
                },
                BFPCommand::Loop(body) => self.run_bfp_loop(body)?,
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
            let optimized_program = BFPProgram::compile(&program);
            machine.run_optimized(&optimized_program);
        }
    }
}
