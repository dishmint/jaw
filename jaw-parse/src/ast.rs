use serde::Serialize;

use crate::token::Span;

#[derive(Debug, Clone, Serialize)]
pub struct Source {
    pub items: Vec<TopLevel>,
}

#[derive(Debug, Clone, Serialize)]
pub enum TopLevel {
    Variable(Variable),
    Function(Function),
    Comment(Comment),
    Step(Step),
    Text(TextNode),
}

#[derive(Debug, Clone, Serialize)]
pub struct Variable {
    pub name: String,
    pub description: String,
    pub decorators: Vec<Decorator>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct InlineAssign {
    pub name: String,
    pub description: String,
    pub value: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct Function {
    pub name: String,
    pub decorators: Vec<Decorator>,
    pub args: Vec<InlineAssign>,
    pub body: CodeBlock,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeBlock {
    pub items: Vec<BlockItem>,
}

#[derive(Debug, Clone, Serialize)]
pub enum BlockItem {
    InlineAssign(InlineAssign),
    Step(Step),
    ComplexCond(ComplexCond),
    Loop(Loop),
    Parallel(Parallel),
    Comment(Comment),
    Log(Log),
    Return(Return),
}

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub number: u32,
    pub expression: Expression,
    pub decorators: Vec<Decorator>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum Expression {
    Code(String),
    Conditional(Conditional),
}

#[derive(Debug, Clone, Serialize)]
pub struct Conditional {
    pub branches: Vec<CondBranch>,
    pub else_branch: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct CondBranch {
    pub condition: String,
    pub consequence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplexCond {
    pub step_number: u32,
    pub condition: String,
    pub true_branch: String,
    pub false_branch: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct Loop {
    pub expr: LoopExpr,
    pub body: CodeBlock,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum LoopExpr {
    While(String),
    ForEach {
        vars: Vec<String>,
        iterable: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Parallel {
    pub body: CodeBlock,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct Return {
    pub value: String,
    pub decorators: Vec<Decorator>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct Log {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum Comment {
    Code { text: String, span: Span },
    General { text: String, span: Span },
}

#[derive(Debug, Clone, Serialize)]
pub struct Decorator {
    pub name: String,
    pub value: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextNode {
    pub text: String,
    pub span: Span,
}
