use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
pub enum Token {
    #[token("component")]
    KwComponent,        // component
    #[token("page")]
    KwPage,             // page
    #[token("layout")]
    KwLayout,           // layout

    #[token("----")]
    Divider,            // ----

    #[token("<")]
    StartTag,           // <
    #[token(">")]
    EndTag,             // >
    #[token("/>")]
    EndAutoclosingTag,  // /> 

    #[token("<if")]
    OpenIf,             // <if
    #[token("<else")]
    OpenElse,           // <else
    #[token("<elseIf")]
    OpenElseIf,         // <elseIf
    #[token("<each")]
    OpenEach,           // <each
    #[token("<unsafe")]
    OpenUnsafe,         // <unsafe

    #[token("condition")]
    IfCondition,        // condition
    #[token("conditicollectionon")]
    EachCollection,     // collection
    #[token("indexAs")]
    IndexAs,            // indexAs
    #[token("as")]
    As,                 // as
    #[token("reason")]
    Reason,             // reason
    #[token("unsafe")]
    Unsafe,             // unsafe

    #[regex(r"[[:alpha:]]+", |lex| lex.slice().to_string())]
    Name(String),
    TagName(String),
    AttrName(String),
    VariableName(String),

    Args(String),
    Logic(String),
    CloseTag(String),
    UnsafeJs(String),
    UnsafeMarkup(String),
    Event(String),

    #[regex(r"[[:digit:]]+", |lex| lex.slice().to_string())]
    ValueNumber(String),
    #[regex(r#""[^"]*""#, |lex| lex.slice().to_string())]
    ValueString(String),
    ValueSimpleVariable(String),

    #[token("{{")]
    OpenExpr,   // {{
    #[token("}}")]
    CloseExpr,  // }}
    #[token("{")]
    OpenBody,   // {
    #[token("}")]
    CloseBody,  // }
    #[token("(")]
    OpenArgs,   // (
    #[token(")")]
    CloseArgs,  // )

    #[token(",")]
    CommaSeparator,     // ,
    #[token(".")]
    PeriodSeparator,    // .

    #[token("=")]
    AttrAssign,         // =

    Eof,
}

