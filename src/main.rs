#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]

use std::collections::VecDeque;

const STACK_MAX: u32 = 256;

enum OpCode
{
    OP_NOP = 100,
    OP_RETURN = 0,
    OP_CONST = 1,
    OP_AND = 2,
    OP_OR = 3,
    OP_ADD = 4
}

impl OpCode
{
    fn from_u8(c: u8) -> OpCode
    {
        match c
        {
            0 => OpCode::OP_RETURN,
            1 => OpCode::OP_CONST,
            2 => OpCode::OP_AND,
            3 => OpCode::OP_OR,
            4 => OpCode::OP_ADD,
            _ => OpCode::OP_NOP
        }
    }
}

#[derive(Clone, Copy)]
enum PrimType
{
    Double(f64),
    Integer(i64),
    Unknown
}

struct Pool
{
    data: VecDeque<PoolItem>,
    size: usize
}

struct PoolItem
{
    data: PrimType,
    index: usize
}

impl Pool
{
    fn new() -> Pool
    {
        Pool {
            data: VecDeque::new(),
            size: 0
        }
    }
}

struct Chunk
{
    code: Vec<u8>,
    size: usize,
    const_pool: Pool,
}

impl Chunk
{
    fn new() -> Chunk
    {
        Chunk {
            code: Vec::new(),
            size: 0,
            const_pool: Pool::new()
        }
    }

    fn write(&mut self, byte: OpCode)
    {
        self.code.push(byte as u8);
        self.size += 1;
    }

    fn write_const_int(&mut self, val: i64)
    {
        self.write(OpCode::OP_CONST);
        self.const_pool.data.push_back(PoolItem { 
            data: PrimType::Integer(val),
            index: self.const_pool.size
        });
        self.const_pool.size += 1;
    }

    fn write_const_double(&mut self, val: f64)
    {
        self.write(OpCode::OP_CONST);
        self.const_pool.data.push_back(PoolItem {
            data: PrimType::Double(val),
            index: self.const_pool.size
        });
        self.const_pool.size += 1;
    }

    fn read_const(&mut self) -> PrimType
    {
        if self.const_pool.size == 0 {()}

        self.const_pool.size -= 1;
        if let Some(value) = &self.const_pool.data.pop_front()
        {
            value.data
        }
        else {
            PrimType::Unknown
        }
    }

    fn dump(&self)
    {
        let mut pool_off: usize = 0;
        let mut code_off: usize = 0;
        let size: usize = self.size;
        while code_off < size
        {
            self._dump_instr(&mut code_off, &mut pool_off);
        }
    }

    fn _dump_instr(&self, code_off: &mut usize, pool_off: &mut usize)
    {
        let instr: u8 = self.code[*code_off];
        let opcode = OpCode::from_u8(instr);
        match opcode
        {
            OpCode::OP_RETURN => { self._dump_simple_instr("OP_RETURN", code_off); },
            OpCode::OP_NOP => { self._dump_simple_instr("OP_NOP", code_off); },
            OpCode::OP_CONST => { *code_off += 1; *pool_off += 1; },
            _ => {
                println!("{instr}");
                println!("Invalid Opcode");
                std::process::exit(1);
            }
        }
    }

    fn _dump_simple_instr(&self, name: &str, code_off: &mut usize)
    {
        println!("{:0>4} {}", format!("{:x}", code_off), name);
        *code_off += 1;
    }
}

enum InterpResult
{
    COMPILE_ERROR = 0,
    RUNTIME_ERROR = 1,
    OK = 2
}

struct VirtMac
{
    chunk: Chunk,
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

    fn interpret(&mut self) -> InterpResult
    {
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
                    PrimType::Double(_) | PrimType::Unknown => {
                        ok = false;
                        println!("");
                        0
                    },
                    PrimType::Integer(value) => *value
                };
                let bvalue = match bb {
                    PrimType::Double(_) | PrimType::Unknown => {
                        ok = false;
                        println!("");
                        0
                    },
                    PrimType::Integer(value) => *value
                };
                self._perform_logical_op(instr, avalue, bvalue);
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

                if avalue_double && bvalue_double
                {
                    self._perform_arithmetic_op_double(instr, avalue_f, bvalue_f);
                }
                else if avalue_double && !bvalue_double
                {
                    self._perform_arithmetic_op_double(instr, avalue_f, bvalue_i as f64);
                }
                else if !avalue_double && bvalue_double
                {
                    self._perform_arithmetic_op_double(instr, avalue_i as f64, bvalue_f);
                }
                else {
                    self._perform_arithmetic_op_int(instr, avalue_i, bvalue_i);
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
    let mut c = Chunk::new();
    c.write_const_double(2.4);
    c.write_const_int(2);
    c.write(OpCode::OP_ADD);
    c.write(OpCode::OP_RETURN);

    let mut vm = VirtMac::new(c);
    vm.interpret();
    vm._dump_stack();
}