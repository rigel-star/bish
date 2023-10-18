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

use std::borrow::BorrowMut;
use std::collections::HashMap;

use crate::interp::ast;
use crate::scanner;

pub struct Interpreter {
    pub var_env: HashMap<String, ast::LiteralType>
}

impl Interpreter {
    #[inline]
    pub fn init() -> Self {
        Self {
            var_env: HashMap::<String, ast::LiteralType>::new()
        }
    }

    #[inline]
    pub fn interpret(&mut self, exprs: &mut [&mut dyn ast::Expr]) {
        for expr in exprs {
            expr.accept(self);
        }
    }
}

impl ast::ExprVisitor for Interpreter {
    fn visit_literal_expr(&mut self, lit_expr: &mut ast::LiteralExpr) -> ast::LiteralType {
        match lit_expr.value.clone() {
            ast::LiteralType::Integer(ivalue) => ast::LiteralType::Integer(ivalue),
            ast::LiteralType::Float(fvalue) => ast::LiteralType::Float(fvalue),
            ast::LiteralType::Boolean(bvalue) => ast::LiteralType::Boolean(bvalue),
            ast::LiteralType::CString(svalue) => ast::LiteralType::CString(svalue),
            _ => ast::LiteralType::Unknown()
        } 
    }

    fn visit_assign_expr(&mut self, asi_expr: &mut ast::AssignExpr) -> ast::LiteralType {
        let val = asi_expr.value.accept(self);
        self.var_env.insert(asi_expr.name.clone(), val.clone()); 
        val
    }
}