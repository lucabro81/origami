/* component, page or layout*/
%token COMPONENT

%token DIVIDER "----"
%token NAME
%token LOGIC

%token START_TAG "<"
%token END_TAG ">"
%token END_AUTOCLOSING_TAG " />"
%token CLOSE_TAG

%token TAG_NAME

%token OPEN_IF
%token IF_CONDITION "condition"
%token OPEN_ELSE
%token OPEN_ELSE_IF

%token OPEN_EACH
%token EACH_COLLECTION "collection"
%token AS "as"
%token INDEX_AS "indexAs"

%token OPEN_UNSAFE
%token REASON "reason"

%token UNSAFE "unsafe"
%token UNSAFE_JS
%token UNSAFE_MARKUP

%token ATTR_NAME
%token EVENT /* can be @eventName */
%token VALUE_NUMBER /* simple number like 123 or 432,21 */
%token VALUE_STRING /* strings between double quotes, like "this is a string" or "123" */
%token VALUE_SIMPLE_VARIABLE /* something without spaces and between {{}}, like {{someVariable}} */

%token VARIABLE_NAME /* a simple string without spaces and without any delimeter like someVariable */

%token OPEN_EXPR "{{"
%token CLOSE_EXPR "}}"

%token OPEN_BODY "{"
%token CLOSE_BODY "}"

%token OPEN_ARGS "("
%token CLOSE_ARGS ")"

%token COMMA_SEPARATOR ","
%token PERIOD_SEPARATOR "."

%token ATTR_ASSIGN "="
%token TYPE_ASSIGN ":"

%%

program:
    %empty
  | program component
  ;

component:
    header OPEN_BODY LOGIC DIVIDER template CLOSE_BODY
  ;

header:
    COMPONENT NAME OPEN_ARGS props CLOSE_ARGS
  | COMPONENT NAME
  ;

props:
    prop
  | props COMMA_SEPARATOR prop
  ;

prop:
    NAME TYPE_ASSIGN NAME
  ;

template:
  children
  ;

child:
  VALUE_NUMBER
  | VALUE_STRING
  | expr
  | autoclosing_tag
  | open_tag children CLOSE_TAG
  | if_condition
  | each_tag
  | unsafe_block
  ;

each_tag:
    OPEN_EACH EACH_COLLECTION ATTR_ASSIGN expr AS ATTR_ASSIGN VALUE_SIMPLE_VARIABLE INDEX_AS ATTR_ASSIGN VALUE_SIMPLE_VARIABLE END_TAG
      children
    CLOSE_TAG
  | OPEN_EACH EACH_COLLECTION ATTR_ASSIGN expr AS ATTR_ASSIGN VALUE_SIMPLE_VARIABLE END_TAG
      children
    CLOSE_TAG
  ;

if_condition:
    OPEN_IF IF_CONDITION ATTR_ASSIGN expr END_TAG 
      children 
      else_if_block
      OPEN_ELSE 
        children
      CLOSE_TAG
    CLOSE_TAG
  | OPEN_IF IF_CONDITION ATTR_ASSIGN expr END_TAG 
      children 
      else_if_block
    CLOSE_TAG
  ;

else_if_block: 
  %empty 
  | else_if_block 
    OPEN_ELSE_IF IF_CONDITION ATTR_ASSIGN expr END_TAG
      children
    CLOSE_TAG
  ;

unsafe_block:
    OPEN_UNSAFE REASON ATTR_ASSIGN VALUE_STRING END_TAG
      UNSAFE_JS
    CLOSE_TAG
  | OPEN_UNSAFE REASON ATTR_ASSIGN VALUE_STRING END_TAG
      UNSAFE_MARKUP
    CLOSE_TAG
  ;

children:
  %empty
  | children child
  ;

autoclosing_tag:
  start_tag END_AUTOCLOSING_TAG
  ;

open_tag:
  start_tag END_TAG
  ;

start_tag:
  START_TAG TAG_NAME attributes
  ;

attributes: 
    %empty
  | attributes attribute
  ;

attribute: 
    ATTR_NAME ATTR_ASSIGN VALUE_NUMBER
  | ATTR_NAME ATTR_ASSIGN VALUE_STRING
  | ATTR_NAME ATTR_ASSIGN expr
  | ATTR_NAME ATTR_ASSIGN unsafe_value
  | EVENT ATTR_ASSIGN expr
  ;

expr:
  OPEN_EXPR expr_value CLOSE_EXPR
  ;

expr_value:
    VARIABLE_NAME
  | expr_value PERIOD_SEPARATOR VARIABLE_NAME
  ;

unsafe_value:
    UNSAFE OPEN_ARGS VALUE_NUMBER COMMA_SEPARATOR VALUE_STRING CLOSE_ARGS
  | UNSAFE OPEN_ARGS VALUE_STRING COMMA_SEPARATOR VALUE_STRING CLOSE_ARGS
  ;