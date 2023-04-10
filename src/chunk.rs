#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]

use std::collections::VecDeque;

#[derive(Debug)]
pub enum OpCode
{
    OP_NOP = 100,
    OP_RETURN = 0,
    OP_CONST = 1,
    OP_AND = 2,
    OP_OR = 3,
    OP_ADD = 4,
    OP_NEGATE = 5,
    OP_SUBTRACT = 6,
    OP_MULTIPLY = 7,
    OP_DIVIDE = 8,
    OP_TRUE = 9,
    OP_FALSE = 10,
    OP_NIL = 11
}

impl OpCode
{
    pub fn from_u8(c: u8) -> OpCode
    {
        match c
        {
            0 => OpCode::OP_RETURN,
            1 => OpCode::OP_CONST,
            2 => OpCode::OP_AND,
            3 => OpCode::OP_OR,
            4 => OpCode::OP_ADD,
            5 => OpCode::OP_NEGATE,
            6 => OpCode::OP_SUBTRACT,
            7 => OpCode::OP_MULTIPLY,
            8 => OpCode::OP_DIVIDE,
            9 => OpCode::OP_TRUE,
            10 => OpCode::OP_FALSE,
            11 => OpCode::OP_NIL,
            _ => OpCode::OP_NOP
        }
    }
}

pub struct Chunk
{
    pub code: Vec<u8>,
    pub size: usize,
    pub const_pool: Pool,
}

impl Chunk
{
    pub fn new() -> Chunk
    {
        Chunk {
            code: Vec::new(),
            size: 0,
            const_pool: Pool::new()
        }
    }

    pub fn write(&mut self, byte: OpCode)
    {
        self.code.push(byte as u8);
        self.size += 1;
    }

    pub fn write_const_int(&mut self, val: i64)
    {
        self.write(OpCode::OP_CONST);
        self.const_pool.data.push_back(PoolItem { 
            data: PrimType::Integer(val),
            index: self.const_pool.size
        });
        self.const_pool.size += 1;
    }

    pub fn write_const_double(&mut self, val: f64)
    {
        self.write(OpCode::OP_CONST);
        self.const_pool.data.push_back(PoolItem {
            data: PrimType::Double(val),
            index: self.const_pool.size
        });
        self.const_pool.size += 1;
    }

    pub fn read_const(&mut self) -> PrimType
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

    pub fn dump(&self)
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

#[derive(Clone, Copy)]
pub enum PrimType
{
    Double(f64),
    Integer(i64),
    Boolean(bool),
    Nil,
    Unknown
}

pub struct Pool
{
    pub data: VecDeque<PoolItem>,
    pub size: usize
}

pub struct PoolItem
{
    pub data: PrimType,
    pub index: usize
}

impl Pool
{
    pub fn new() -> Pool
    {
        Pool {
            data: VecDeque::new(),
            size: 0
        }
    }
}