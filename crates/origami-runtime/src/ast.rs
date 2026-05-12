use miette::SourceSpan;

#[derive(Debug, PartialEq)]
pub struct Prop {
    pub name: String,
    pub type_str: String,
}

#[derive(Debug, PartialEq)]
pub enum Declaration {
    Component { name: String, props: Vec<Prop>, body: Body },
    Page { name: String, props: Vec<Prop>, body: Body },
    Layout { name: String, body: Body },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleExpression {
    Var(String),
    Dot(Box<SimpleExpression>, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Static {
    NumberInt(i64),
    NumberFloat(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    Literal(Static),
    Dynamic(SimpleExpression),
    UnsafeValue { value: Static, reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attr {
    pub name: String,
    pub value: AttrValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentNode {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub children: Vec<Node>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextNode {
    pub value: String,
    pub span: SourceSpan,
}

/// Interpolation of a TypeScript expression in the template: `{{expr}}`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionNode {
    pub value: SimpleExpression,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralNode {
    pub value: Static,
    pub span: SourceSpan,
}

/// Conditional node `<if condition={expr}>…</if>`.
#[derive(Debug, Clone, PartialEq)]
pub struct IfNode {
    pub condition: SimpleExpression,
    pub then_children: Vec<Node>,
    pub else_if_children: Vec<IfNode>,
    pub else_child: Option<Vec<Node>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EachNode {
    pub collection: SimpleExpression,
    pub alias: String,
    pub index_alias: Option<SimpleExpression>,
    pub children: Vec<Node>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnsafeNode {
    pub reason: String,
    pub children: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotNode {
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// Closed-vocabulary component (e.g. `<Column>`, `<Text />`).
    Component(ComponentNode),
    /// Static text.
    Text(TextNode),
    /// Expression interpolation `{{expr}}`.
    Expr(ExpressionNode),
    /// Conditional `<if>`.
    If(IfNode),
    /// Iteration `<each>`.
    Each(EachNode),
    /// Unsafe escape-hatch block `<unsafe>`.
    Unsafe(UnsafeNode),
    /// Placeholder slot for injecting children.
    Slot(SlotNode),
    Literal(LiteralNode),
}

#[derive(Debug, PartialEq)]
pub struct Body {
    pub logic_block: String,
    pub template: Vec<Node>,
}

#[derive(Debug, PartialEq)]
pub struct OriFile {
    pub declarations: Vec<Declaration>,
}
