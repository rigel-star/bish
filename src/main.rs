#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]

pub mod scanner;
pub mod compiler;
pub mod chunk;
use chunk::{Chunk, PrimType, OpCode};

const STACK_MAX: u32 = 256;

enum InterpResult
{
    COMPILE_ERROR = 0,
    RUNTIME_ERROR = 1,
    INTERPRET_COMPILE_ERROR = 2,
    OK = 3
}

struct VirtMac
{
    chunk: chunk::Chunk,
    ip: usize,
    stack: Vec<PrimType>
}

impl VirtMac
{
    fn new(chunk: Chunk) -> VirtMac
    {
        VirtMac {
            chunk,
            ip: 0,
            stack: Vec::new()
        }
    }

    fn stack_push(&mut self, val: PrimType)
    {
        self.stack.push(val);
    }

    fn stack_pop(&mut self) -> PrimType
    {
        if let Some(value) = &self.stack.pop()
        {
            *value
        }
        else 
        {
            PrimType::Unknown
        }
    }

    fn _dump_stack(&self)
    {
        let mut idx: usize = self.stack.len();
        for i in (0..idx).rev()
        {
            match self.stack[i]
            {
                PrimType::Integer(value) => {
                    println!("[{value}]");
                },
                PrimType::Double(value) => {
                    println!("[{value}]");
                },
                PrimType::Unknown => {
                    println!("[UNKNOWN]");
                }
            }
        }
    }

    fn compile(&mut self, source: &str) -> InterpResult
    {
        let mut s: scanner::Scanner = scanner::Scanner::new(String::from(source));
        let mut tokens: Vec<scanner::Token> = s.start_scan();
        let mut parser: compiler::Parser = compiler::Parser::new(&tokens, &mut self.chunk);
        parser.compile();
        InterpResult::OK
    }

    fn interpret(&mut self, source: &str) -> InterpResult
    {
        let compiler_status: InterpResult = self.compile(source);
        match compiler_status
        {
            InterpResult::INTERPRET_COMPILE_ERROR =>
            {
                println!("Compile error!");
                std::process::exit(1);
            },
            _ => ()
        }

        if self.chunk.size < 1
        {
            return InterpResult::OK;
        }

        let mut counter: usize = 0;
        loop
        {
            { 
                let code: u8 = self.chunk.code[counter];
                self._interpret_instr(code); 
            }
            counter += 1;
            if counter == self.chunk.code.len()
            {
                break;
            }
        }
        InterpResult::OK
    }

    fn _interpret_instr(&mut self, i: u8)
    {
        let instr: OpCode = OpCode::from_u8(i);
        match instr
        {
            OpCode::OP_RETURN => { return; },
            OpCode::OP_NOP => (),
            OpCode::OP_CONST => {
                let con = &self.chunk.read_const();
                match con
                {
                    PrimType::Double(value) =>
                    {
                        self.stack_push(PrimType::Double(*value));
                    },
                    PrimType::Integer(value) => {
                        self.stack_push(PrimType::Integer(*value));
                    },
                    PrimType::Unknown => {
                        println!("PANIC: Unknown value type in constant pool!");
                        std::process::exit(1);
                    }
                }
            },
            OpCode::OP_AND | OpCode::OP_OR | OpCode::OP_ADD => {
                self._interpret_binary_instr(instr);
            },
            _ => ()
        }
    }

    fn _interpret_binary_instr(&mut self, instr: OpCode)
    {
        let aa: &PrimType = &self.stack_pop();
        let bb: &PrimType = &self.stack_pop();
        let mut ok: bool = true;

        match instr 
        {
            OpCode::OP_AND | OpCode::OP_OR => {
                let avalue = match aa {
                    PrimType::Double(_) | PrimType::Unknown => {
                        ok = false;
                        0
                    },
                    PrimType::Integer(value) => *value
                };
                let bvalue = match bb {
                    PrimType::Double(_) | PrimType::Unknown => {
                        ok = false;
                        0
                    },
                    PrimType::Integer(value) => *value
                };

                match ok {
                    true => { self._perform_logical_op(instr, avalue, bvalue); }
                    false => { 
                        println!("{}", "Unsupported tyepes for this operation");
                        std::process::exit(3);
                    }
                }
            },
            OpCode::OP_ADD => {
                let mut avalue_double: bool = false;
                let mut bvalue_double: bool = false;
                let mut avalue_f: f64 = 0.0;
                let mut bvalue_f: f64 = 0.0;
                
                let mut avalue_i: i64 = 0;
                let mut bvalue_i: i64 = 0;

                match aa {
                    PrimType::Double(value) => {
                        avalue_double = true;
                        avalue_f = *value;
                    },
                    PrimType::Integer(value) => {
                        avalue_i = *value;
                    },
                    _ => {();}
                };

                match bb {
                    PrimType::Double(value) => {
                        bvalue_double = true;
                        bvalue_f = *value;
                    },
                    PrimType::Integer(value) => {
                        bvalue_i = *value;
                    },
                    _ => {();}
                };

                match (avalue_double, bvalue_double)
                {
                    (true, true) => self._perform_arithmetic_op_double(instr, avalue_f, bvalue_f),
                    (true, false) => self._perform_arithmetic_op_double(instr, avalue_f, bvalue_i as f64),
                    (false, true) => self._perform_arithmetic_op_double(instr, avalue_i as f64, bvalue_f),
                    (false, false) => self._perform_arithmetic_op_int(instr, avalue_i, bvalue_i)
                }
            },
            _ => ()
        }
    }

    fn _perform_logical_op(&mut self, instr: OpCode, avalue: i64, bvalue: i64)
    {
        match instr 
        {
            OpCode::OP_AND => self.stack_push(PrimType::Integer(avalue & bvalue)),
            OpCode::OP_OR => self.stack_push(PrimType::Integer(avalue | bvalue)),
            _ => ()
        }
    }

    fn _perform_arithmetic_op_double(&mut self, instr: OpCode, avalue: f64, bvalue: f64)
    {
        match instr
        {
            OpCode::OP_ADD => {
                self.stack_push(PrimType::Double(avalue + bvalue));
            },
            _ => ()
        }
    }
    
    fn _perform_arithmetic_op_int(&mut self, instr: OpCode, avalue: i64, bvalue: i64)
    {
        match instr
        {
            OpCode::OP_ADD => {
                self.stack_push(PrimType::Integer(avalue + bvalue));
            },
            _ => ()
        }
    }
}

fn main() {
    let mut c: Chunk = Chunk::new();
    let mut vm: VirtMac = VirtMac::new(c);
    vm.interpret("5+1+3+");
    vm._dump_stack();
}
