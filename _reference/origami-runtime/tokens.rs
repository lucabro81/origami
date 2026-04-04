
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Component(String),
    Divider,
    StartTag,
    EndTag,
    EndAutoclosingTag,
    OpenIf,
    OpenElse,
    OpenElseIf,
    OpenEach,
    OpenUnsafe,

    IfCondition,
    EachCollection,
    IndexAs,
    As,
    Reason,
    Unsafe,

    Name(String),
    Args(String),
    Logic(String),
    CloseTag(String),
    TagName(String),
    UnsafeJs(String),
    UnsafeMarkup(String),
    AttrName(String),
    Event(String),
    ValueNumber(String),
    ValueString(String),
    ValueSimpleVariable(String),
    VariableName(String),

    OpenExpr, // {{
    CloseExpr, // }}
    OpenBody, // {
    CloseBody, // }
    OpenArgs, // (
    CloseArgs, // )

    CommaSeparator,
    PeriodSeparator,

    AttrAssign,

    Eof,
}

