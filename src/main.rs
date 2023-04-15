#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]

pub mod scanner;
pub mod compiler;
pub mod chunk;
use chunk::{Chunk, PrimType, OpCode};

use std::collections::HashMap;

const STACK_MAX: u32 = 256;

#[derive(PartialEq, Eq)]
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
    stack: Vec<PrimType>,
    globals: HashMap<String, PrimType>
}

impl VirtMac
{
    fn new(chunk: Chunk) -> VirtMac
    {
        VirtMac {
            chunk,
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::<String, PrimType>::new()
        }
    }

    fn stack_push(&mut self, val: PrimType)
    {
        // println!("DEBUG[stack_push]: PrimType = {:?}", &val);
        self.stack.push(val);
    }

    fn stack_pop(&mut self) -> PrimType
    {
        if let Some(value) = &self.stack.pop()
        {
            value.clone()
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
            match self.stack[i].clone()
            {
                PrimType::Integer(value) => println!("[{value}]"),
                PrimType::Double(value) => println!("[{value}]"),
                PrimType::Boolean(value) => println!("[{}]", if value { "sahi(true)" } else { "galat(false)" } ),
                PrimType::CString(len, data) => println!("[{data}({len})]"),
                PrimType::Nil => println!("[nil]"),
                PrimType::Unknown => println!("[UNKNOWN]")
            }
        }
    }

    fn compile(&mut self, source: &str) -> InterpResult
    {
        let mut s: scanner::Scanner = scanner::Scanner::new(String::from(source));
        let tokens: Vec<scanner::Token> = s.start_scan();
        let mut parser: compiler::Parser = compiler::Parser::new(&tokens, &mut self.chunk);
        if parser.compile() == compiler::CompilationResult::Ok { InterpResult::OK }
        else { InterpResult::COMPILE_ERROR }
    }

    fn interpret(&mut self, source: &str) -> InterpResult
    {
        if InterpResult::COMPILE_ERROR == self.compile(source)
        {
            println!("compile error. terminated.");
            std::process::exit(1);
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
            OpCode::OP_RETURN => { },
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
                    PrimType::CString(len, data) => {
                        self.stack_push(PrimType::CString(*len, data.clone()));
                    },
                    PrimType::Unknown => {
                        println!("PANIC: Unknown value type in constant pool!");
                        std::process::exit(1);
                    },
                    _ => ()
                }
            },
            OpCode::OP_TRUE => self.stack_push(PrimType::Boolean(true)),
            OpCode::OP_FALSE => self.stack_push(PrimType::Boolean(false)),
            OpCode::OP_NIL => self.stack_push(PrimType::Nil),
            OpCode::OP_AND | 
            OpCode::OP_OR | 
            OpCode::OP_ADD | 
            OpCode::OP_SUBTRACT | 
            OpCode::OP_MULTIPLY |
            OpCode::OP_DIVIDE | 
            OpCode::OP_EQ_EQ |
            OpCode::OP_GT |
            OpCode::OP_LT => self._interpret_binary_instr(instr),
            OpCode::OP_NEGATE => self._perform_negate_op(),
            OpCode::OP_NOT => self._perform_not_op(),
            OpCode::OP_PRINT => self._interpret_print_stmt(),
            OpCode::OP_POP => { self.stack_pop(); },
            OpCode::OP_DEF_GLOBAL => {
                let name: PrimType = self.chunk.read_const();
                let value: PrimType = self.stack_pop();
                self._define_global_var(name, value);
                self.stack_pop();
            },
            OpCode::OP_LOAD_GLOBAL => {
                let name: PrimType = self.chunk.read_const();
                #[allow(clippy::single_match)]
                match name {
                    PrimType::CString(_, value) => {
                        let value = self.globals.get(&value);
                        self.stack_push(value.unwrap().clone());
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }

    fn _define_global_var(&mut self, name: PrimType, value: PrimType)
    {
        match name {
            PrimType::CString(_, var) => self.globals.insert(var, value),
            PrimType::Double(_) => unimplemented!(),
            PrimType::Integer(_) => unimplemented!(),
            PrimType::Boolean(_) => unimplemented!(),
            PrimType::Nil => unimplemented!(),
            PrimType::Unknown => unimplemented!()
        };
    }

    fn _interpret_print_stmt(&mut self)
    {
        let value: &PrimType = &self.stack_pop();
        match value 
        {
            PrimType::CString(len, value) => println!("{}", value),
            PrimType::Double(value) => println!("{}", value),
            PrimType::Integer(value) => println!("{}", value),
            PrimType::Boolean(value) => println!("{}", if *value { "sahi" } else { "galat" }),
            PrimType::Nil => println!("nil"),
            _ => {
                println!("Can't print");
                std::process::exit(10);
            }
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
                    PrimType::Integer(value) => *value,
                    _ => {
                        ok = false;
                        0
                    }
                };
                let bvalue = match bb {
                    PrimType::Integer(value) => *value,
                    _ => {
                        ok = false;
                        0
                    }
                };

                match ok {
                    true => { self._perform_logical_op(instr, avalue, bvalue); }
                    false => { 
                        println!("Unsupported types for this operation");
                        std::process::exit(3);
                    }
                }
            },
            OpCode::OP_ADD |
            OpCode::OP_SUBTRACT |
            OpCode::OP_MULTIPLY |
            OpCode::OP_DIVIDE => {
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
                    _ => {}
                };

                match bb {
                    PrimType::Double(value) => {
                        bvalue_double = true;
                        bvalue_f = *value;
                    },
                    PrimType::Integer(value) => {
                        bvalue_i = *value;
                    },
                    _ => {}
                };

                match (avalue_double, bvalue_double)
                {
                    (true, true) => self._perform_arithmetic_op_double(instr, avalue_f, bvalue_f),
                    (true, false) => self._perform_arithmetic_op_double(instr, avalue_f, bvalue_i as f64),
                    (false, true) => self._perform_arithmetic_op_double(instr, avalue_i as f64, bvalue_f),
                    (false, false) => self._perform_arithmetic_op_int(instr, avalue_i, bvalue_i)
                }
            },
            OpCode::OP_LT |
            OpCode::OP_GT |
            OpCode::OP_EQ_EQ => {
                self._perform_relational_op(aa, bb, instr);
            },
            _ => ()
        }
    }

    fn _perform_not_op(&mut self)
    {
        let value: &PrimType = &self.stack_pop();
        match value 
        {
            PrimType::Integer(value) => self.stack_push(PrimType::Boolean(*value == 0)),
            PrimType::Boolean(cond) => self.stack_push(PrimType::Boolean(!cond)),
            _ => {
                println!("Type error: can't apply 'chhaina' operator on type '{}'", PrimType::name(value));
                std::process::exit(9);
            }
        }
    }

    fn _perform_negate_op(&mut self)
    {
        let value: &PrimType = &self.stack_pop();
        match value 
        {
            PrimType::Integer(value) => self.stack_push(PrimType::Integer(-*value)),
            PrimType::Double(value) => self.stack_push(PrimType::Double(-*value)),
            _ => {
                println!("Type error: can't apply negate(-) operator on type '{}'", PrimType::name(value));
                std::process::exit(8);
            }
        }
    }

    fn _perform_relational_op(&mut self, val1: &PrimType, val2: &PrimType, instr: OpCode)
    {
        let result: bool = match instr 
        {
            OpCode::OP_GT => self._relational_op_gt(val1, val2),
            OpCode::OP_LT => self._relational_op_lt(val1, val2),
            OpCode::OP_EQ_EQ => self._relational_op_eq_eq(val1, val2),
            _ => false
        };
        self.stack_push(PrimType::Boolean(result));
    }

    fn _relational_op_eq_eq(&mut self, val1: &PrimType, val2: &PrimType) -> bool
    {
        match (val1, val2)
        {
            (PrimType::Integer(a), PrimType::Integer(b)) => (a == b),
            (PrimType::Double(a), PrimType::Double(b)) => (a == b),
            (PrimType::Boolean(cond1), PrimType::Boolean(cond2)) => (cond1 == cond2),
            (PrimType::CString(len1, val1), PrimType::CString(len2, val2)) => (val1 == val2),
            _ => {
                self.panic_type_error("barabar", &PrimType::name(val1), &PrimType::name(val2));
                false 
            }
        }
    }

    fn _relational_op_gt(&mut self, val1: &PrimType, val2: &PrimType) -> bool
    {
        match (val1, val2)
        {
            (PrimType::Integer(a), PrimType::Integer(b)) => (b > a),
            (PrimType::Double(a), PrimType::Double(b)) => (b > a),
            _ => {
                self.panic_type_error("thulo", &PrimType::name(val1), &PrimType::name(val2));
                false
            }
        }
    }

    fn _relational_op_lt(&mut self, val1: &PrimType, val2: &PrimType) -> bool
    {
        match (val1, val2)
        {
            (PrimType::Integer(a), PrimType::Integer(b)) => (b < a),
            (PrimType::Double(a), PrimType::Double(b)) => (b < a),
            _ => {
                self.panic_type_error("sano", &PrimType::name(val1), &PrimType::name(val2));
                false
            }
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
            OpCode::OP_SUBTRACT => {
                self.stack_push(PrimType::Double(bvalue - avalue));
            },
            OpCode::OP_DIVIDE => {
                self.stack_push(PrimType::Double(bvalue / avalue));
            },
            OpCode::OP_MULTIPLY => {
                self.stack_push(PrimType::Double(bvalue * avalue));
            }
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
            OpCode::OP_SUBTRACT => {
                self.stack_push(PrimType::Integer(bvalue - avalue));
            },
            OpCode::OP_DIVIDE => {
                self.stack_push(PrimType::Integer((bvalue as f64 / avalue as f64) as i64));
            },
            OpCode::OP_MULTIPLY => {
                self.stack_push(PrimType::Integer(bvalue * avalue));
            }
            _ => ()
        }
    }

    fn panic_type_error(&self, op: &str, type1: &str, type2: &str)
    {
        println!("Type error: '{}' ra '{}' prakar ko value harulai '{}' operator lagauna mildaina.", type1, type2, op);
        std::process::exit(7);
    }
}

use std::{env, fs};

fn main() {
    let _args: Vec<String> = env::args().collect();
    if _args.len() < 2
    {
        println!("Usage: cargo run <file_path>");
        std::process::exit(12);
    }

    let file_path = &_args.get(1usize);
    let source_code: String = match fs::read_to_string(file_path.unwrap()) {
        Ok(content) => content,
        Err(error) => {
            println!("Tapaile diyeko file lai padhna sakiyena.");
            std::process::exit(15);
        }
    };
    let mut c: Chunk = Chunk::new();
    let mut vm: VirtMac = VirtMac::new(c);
    vm.interpret(&source_code);
    vm._dump_stack();
}