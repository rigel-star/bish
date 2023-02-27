#![allow(dead_code)]
#![allow(unused)]

const STACK_MAX: u32 = 256;

enum OpCode
{
    OP_NOP = 0,
    OP_RETURN = 1,
    OP_CONST = 2,
    OP_AND = 3,
    OP_OR = 4
}

impl OpCode
{
    fn from_u8(c: u8) -> OpCode
    {
        match c
        {
            0 => OpCode::OP_NOP,
            1 => OpCode::OP_RETURN,
            2 => OpCode::OP_CONST,
            3 => OpCode::OP_AND,
            4 => OpCode::OP_OR,
            _ => OpCode::OP_NOP
        }
    }
}

enum PrimType
{
    Double(f64),
    Integer(i32)
}

struct Pool
{
    data: Vec<PrimType>,
    size: usize
}

impl Pool
{
    fn new() -> Pool
    {
        Pool {
            data: Vec::new(),
            size: 0
        }
    }
}

struct Chunk
{
    code: Vec<u8>,
    size: usize,
    const_pool: Pool
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

    fn write_const_int(&mut self, val: i32)
    {
        self.write(OpCode::OP_CONST);
        self.const_pool.data.push(PrimType::Integer(val));
        self.const_pool.size += 1;
    }

    fn write_const_double(&mut self, val: f64)
    {
        self.write(OpCode::OP_CONST);
        self.const_pool.data.push(PrimType::Double(val));
        self.const_pool.size += 1;
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
                println!("{}", instr);
                println!("Invalid Opcode");
                std::process::exit(1);
            }
        }
    }

    fn _dump_simple_instr(&self, name: &str, code_off: &mut usize)
    {
        // right justify by 4 and fill with 0
        println!("{:0>4} {}", format!("{:x}", code_off), name);
        *code_off += 1;
    }

    // fn _dump_const_instr(&self, name: &str, code_off: &mut usize, pool_off: &mut usize)
    // {
    //     println!(
    //         "{:0>4} {} {:0>4} {}", 
    //         format!("{:x}", code_off),
    //         name,
    //         format!("{:x}", pool_off),
    //         self.const_pool.data[*pool_off]
    //     );
    //     *code_off += 1;
    //     *pool_off += 1;
    // }
}

enum InterpResult
{
    COMPILE_ERROR = 0,
    RUNTIME_ERROR = 1,
    OK = 2
}

struct VirtMac<'a>
{
    chunk: Chunk,
    ip: usize,
    stack: Vec<&'a PrimType>,
    sp: usize
}

impl<'a> VirtMac<'a>
{
    fn new(chunk: Chunk) -> VirtMac<'a>
    {
        VirtMac {
            chunk: chunk,
            ip: 0,
            stack: Vec::new(),
            sp: 0
        }
    }

    fn reset_stack(&mut self)
    {
        self.sp = 0;
    }

    fn stack_push(&mut self, val: &'a PrimType)
    {
        self.stack.insert(self.sp, val);
        self.sp += 1;
    }

    fn stack_pop(&mut self) -> &PrimType
    {
        let val = &self.stack[self.sp - 1];
        self.sp -= 1;
        val
    }

    fn _dump_stack(&self)
    {
        let mut idx: usize = self.sp;
        for i in (0..idx).rev()
        {
            println!("[{}]", 0);
        }
    }

    fn interpret(&mut self) -> InterpResult
    {
        let mut pool_off: usize = 0;

        loop
        {
            let i = self.chunk.code[self.ip];
            let instr: OpCode = OpCode::from_u8(i);
            match instr
            {
                OpCode::OP_RETURN => { break; },
                OpCode::OP_NOP => (),
                OpCode::OP_CONST => {
                    let cc = self._read_const(&mut pool_off);
                    self.stack_push(cc);
                },
                OpCode::OP_AND | OpCode::OP_OR => {
                    let a = self.stack_pop();
                    let b = self.stack_pop();
                    let mut ok = true;

                    let mut atype = String::from("");
                    let mut avalue: i32 = 0;

                    let mut btype = String::from("");
                    let mut bvalue: i32 = 0;

                    avalue = match a {
                        PrimType::Double(_) => { 
                            ok = false;
                            atype.push_str("float");
                            0
                        },
                        PrimType::Integer(value) => *value
                    };

                    bvalue = match b {
                        PrimType::Double(_) => { 
                            ok = false; 
                            btype.push_str("float");
                            0
                        },
                        PrimType::Integer(value) => *value
                    };

                    if ok
                    {
                        self.stack_push(&PrimType::Integer(avalue & bvalue));
                    }
                    else
                    {
                        println!("Can't perform & on type(s): '{}' and '{}'", atype, btype);
                        std::process::exit(2);
                    }
                }
            }
            self.ip += 1;
        }
        InterpResult::OK
    }

    fn _read_const(&self, off: &mut usize) -> &PrimType
    {
        let data = &self.chunk.const_pool.data[*off];
        *off += 1;
        data
    }
}

fn main() {
    let mut c = Chunk::new();
    c.write_const_int(2);
    c.write_const_int(2);
    c.write(OpCode::OP_AND);
    c.write(OpCode::OP_RETURN);

    let mut vm = VirtMac::new(c);
    vm.interpret();
    vm._dump_stack();
}
