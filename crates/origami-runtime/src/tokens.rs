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
    #[token("<slot />")]
    Slot,         

    #[token("condition")]
    IfCondition,        
    #[token("collection")]
    EachCollection,     
    #[token("indexAs")]
    IndexAs,            
    #[token("as")]
    As,                 
    #[token("reason")]
    Reason,             
    #[token("unsafe")]
    Unsafe,             

    #[regex(r"[[:alpha:]]+", |lex| lex.slice().to_string())]
    Ident(String), // <--

    #[regex(r"</[[:alpha:]]+>", |lex| lex.slice()[2..lex.slice().len()-1].to_string())]
    CloseTag(String),

    #[regex(r"@[[:alpha:]]+", |lex| lex.slice().to_string())]
    Event(String),

    #[regex(r"[0-9]+\.[0-9]+|[0-9]+", |lex| lex.slice().to_string(), priority = 10)]
    ValueNumber(String),
    #[regex(r#""[^"]*""#, |lex| lex.slice().to_string())]
    ValueString(String),

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
    #[token(":")]
    TypeAssign,         // :

    /// Raw JS/TS content between `{` and `----`. Payload filled after lexing.
    #[token("__LOGIC__", |_| String::new())]
    LogicBlock(String),
    /// Raw content inside `<unsafe>...</unsafe>`. Payload unused.
    #[token("__UNSAFE__", |_| String::new())]
    UnsafeBlock(String),

    Eof,
}

