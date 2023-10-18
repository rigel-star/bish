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

use crate::scanner;
use std;

#[derive(Clone, Debug)]
pub enum LiteralType {
    Integer(i32),
    Float(f32),
    CString(String),
    Boolean(bool),
    Unknown()
}

pub trait ExprVisitor {
    fn visit_literal_expr(&mut self, lit_expr: &mut LiteralExpr) -> LiteralType;

    fn visit_assign_expr(&mut self, asi_expr: &mut AssignExpr) -> LiteralType;
}

pub trait Expr {
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> LiteralType;
}

pub struct LiteralExpr {
    pub value: LiteralType
}

impl Expr for LiteralExpr {
    #[inline]
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> LiteralType {
        visitor.visit_literal_expr(self)
    }
}

pub struct AssignExpr<'a> {
    pub name: String,
    pub value: &'a mut dyn Expr
}

impl<'a> Expr for AssignExpr<'a> {
    #[inline]
    fn accept(&mut self, visitor: &mut dyn ExprVisitor) -> LiteralType {
        visitor.visit_assign_expr(self)
    }
}