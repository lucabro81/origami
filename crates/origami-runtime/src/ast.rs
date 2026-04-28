#[derive(Debug, PartialEq)]
pub struct Prop {
    pub name: String,
    pub type_str: String
}

#[derive(Debug, PartialEq)]
pub enum Declaration {
    Component { name: String, props: Vec<Prop>, body: Body },
    Page { name: String, props: Vec<Prop>, body: Body },
    Layout { name: String, body: Body },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleExpression {
    Var(String), // es.: {{simpleValue}}
    Dot(Box<SimpleExpression>, String) // es.: {{book.author.id}}
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
    pub value: AttrValue
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentNode {
    /// Component name (e.g. `"Column"`, `"Text"`).
    pub name: String,
    /// attrs declared on the opening tag.
    pub attrs: Vec<Attr>,
    /// Children: present only if the tag is not self-closing.
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextNode {
    /// The raw text.
    pub value: String,
}

/// Interpolation of a TypeScript expression in the template: `{{expr}}`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionNode {
    pub value: SimpleExpression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralNode {
    pub value: Static,
}

/// Conditional node `<if condition={expr}>…</if>` with optional else and else-if branches.
#[derive(Debug, Clone, PartialEq)]
pub struct IfNode {
    pub condition: String,
    pub then_children: Vec<Node>,
    pub else_if_children: Vec<IfNode>,
    pub else_child: Option<Vec<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EachNode {
    pub collection: SimpleExpression,
    pub alias: String,
    pub index_alias: Option<SimpleExpression>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnsafeNode {
    pub reason: String,
    pub children: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotNode {}

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
    /// Placeholder in a component in which inject children. TBD: it's really needed? can be solved in a react way or simpler?
    Slot(SlotNode),
    Literal(LiteralNode)
}

#[derive(Debug, PartialEq)]
pub struct Body {
    pub logic_block: String,
    pub template: Vec<Node>
}

#[derive(Debug, PartialEq)]
pub struct OriFile { 
    pub declarations: Vec<Declaration>,
}