#[derive(Debug, PartialEq)]
pub struct Prop {
    pub name: String,
    pub type_str: String
}

#[derive(Debug, PartialEq)]
pub enum Declaration {
    Component { name: String, props: Vec<Prop> },
    Page { name: String, props: Vec<Prop> },
    Layout { name: String },
}

#[derive(Debug, PartialEq)]
pub struct OriFile { 
    pub declarations: Vec<Declaration> 
}