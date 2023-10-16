// MIT License
//
// Copyright (c) 2023 Ramesh Poudel
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]
#![allow(clippy::new_without_default)]

use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub enum OpCode {
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
    OP_NIL = 11,
    OP_EQ_EQ = 12,
    OP_LT = 13,
    OP_GT = 14,
    OP_LTE = 15,
    OP_GTE = 16,
    OP_EQ = 17,
    OP_NOT = 18,
    OP_PRINT = 19,
    OP_POP = 20,
    OP_DEF_GLOBAL = 21,
    OP_LOAD_GLOBAL = 22,
    OP_JMP_IF_FALSE = 23,
    OP_ELSE = 24,
    OP_COUNT
}

impl OpCode {
    pub fn from_u8(c: u8) -> OpCode {
        match c {
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
            12 => OpCode::OP_EQ_EQ,
            13 => OpCode::OP_LT,
            14 => OpCode::OP_GT,
            15 => OpCode::OP_LTE,
            16 => OpCode::OP_GTE,
            17 => OpCode::OP_EQ,
            18 => OpCode::OP_NOT,
            19 => OpCode::OP_PRINT,
            20 => OpCode::OP_POP,
            21 => OpCode::OP_DEF_GLOBAL,
            22 => OpCode::OP_LOAD_GLOBAL,
            23 => OpCode::OP_JMP_IF_FALSE,
            24 => OpCode::OP_ELSE,
            _ => OpCode::OP_NOP
        }
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub size: usize,
    pub const_pool: Pool,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            size: 0,
            const_pool: Pool::new()
        }
    }

    #[inline]
    pub fn write(&mut self, byte: OpCode) {
        self.code.push(byte as u8);
        self.size += 1;
    }

    #[inline]
    pub fn write_const_int(&mut self, val: i64) {
        self.write(OpCode::OP_CONST);
        self.write_const(PrimType::Integer(val));
    }

    #[inline]
    pub fn write_cstring(&mut self, value: String) {
        self.write(OpCode::OP_CONST);
        self.write_const(PrimType::CString(value.len(), value));
    }

    #[inline]
    pub fn write_const_double(&mut self, val: f64) {
        self.write(OpCode::OP_CONST);
        self.write_const(PrimType::Double(val));
    }

    #[inline]
    pub fn write_bool(&mut self, cond: bool) {
        self.write(if cond { OpCode::OP_TRUE } else { OpCode::OP_FALSE });
        self.write_const(PrimType::Boolean(cond));
    }

    #[inline]
    pub fn write_nil(&mut self) {
        self.write(OpCode::OP_NIL);
        self.write_const(PrimType::Nil);
    }

    #[inline]
    pub fn write_const(&mut self, prim_type: PrimType) {
        self.const_pool.data.push_back(PoolItem {
            data: prim_type,
            index: self.const_pool.size
        });
        self.const_pool.size += 1;
    }

    pub fn read_const(&mut self) -> PrimType {
        if self.const_pool.size == 0 {}

        self.const_pool.size -= 1;
        if let Some(value) = &self.const_pool.data.pop_front() {
            value.data.clone()
        }
        else {
            PrimType::Unknown
        }
    }

    pub fn dump(&self) {
        let mut pool_off: usize = 0;
        let mut code_off: usize = 0;
        let size: usize = self.size;
        while code_off < size {
            self._dump_instr(&mut code_off, &mut pool_off);
        }
    }

    fn _dump_instr(&self, code_off: &mut usize, pool_off: &mut usize) {
        let instr: u8 = self.code[*code_off];
        let opcode = OpCode::from_u8(instr);
        match opcode {
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

    fn _dump_simple_instr(&self, name: &str, code_off: &mut usize) {
        println!("{:0>4} {}", format!("{:x}", code_off), name);
        *code_off += 1;
    }
}

#[derive(Clone, Debug)]
pub enum PrimType {
    Double(f64),
    Integer(i64),
    Boolean(bool),
    CString(usize, String),
    Nil,
    Unknown
}

impl PrimType {
    pub fn name(typ: &PrimType) -> String {
        match typ {
            PrimType::Double(_) => String::from("float"),
            PrimType::Integer(_) => String::from("int"),
            PrimType::Boolean(_) => String::from("bool"),
            PrimType::CString(_, _) => String::from("string"),
            PrimType::Nil => String::from("nil"),
            _ => String::from("unknown")
        }
    }
}

pub struct Pool {
    pub data: VecDeque<PoolItem>,
    pub size: usize
}

pub struct PoolItem {
    pub data: PrimType,
    pub index: usize
}

impl Pool {
    pub fn new() -> Pool {
        Pool {
            data: VecDeque::<PoolItem>::new(),
            size: 0
        }
    }
}