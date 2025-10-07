🧩 PostgreSQL SQL Syntax & Parsing Guide
1. Overview

A PostgreSQL SQL parser converts a raw SQL string into an Abstract Syntax Tree (AST).
Parsing involves two stages:

Lexical analysis (tokenization) → converts text into tokens.

Syntactic analysis (grammar) → converts tokens into an AST.

2. Lexical Rules

Token categories:

Type	Examples	Notes
Keywords	SELECT, FROM, WHERE, GROUP, ORDER, INSERT, UPDATE	Case-insensitive
Identifiers	table_name, column1, "User"	Double quotes allow reserved words
Literals	'text', 123, TRUE, FALSE, NULL	Strings in single quotes
Operators	=, <>, <, >, <=, >=, +, -, *, /	Standard SQL
Delimiters	,, ., (, ), ;	Structural tokens
Comments	-- line, /* block */	Ignored by parser
Whitespace	space, tab, newline	Used to separate tokens

Identifiers:

Start with a letter or underscore, followed by letters, digits, or underscores.

Quoted identifiers ("Group") preserve case and allow reserved words.

3. Grammar (EBNF-style, PostgreSQL flavor)
statement        := select_stmt (";" | EOF)

select_stmt      := SELECT select_list
                    from_clause?
                    where_clause?
                    group_by_clause?
                    having_clause?
                    order_by_clause?
                    limit_clause?

select_list      := "*" | select_item ("," select_item)*
select_item      := expr (AS? alias)?

from_clause      := FROM table_ref ("," table_ref | join_clause)*
table_ref        := (schema_name ".")? table_name (AS? alias)?
join_clause      := (INNER? | LEFT | RIGHT | FULL) JOIN table_ref ON boolean_expr

where_clause     := WHERE boolean_expr
group_by_clause  := GROUP BY expr ("," expr)*
having_clause    := HAVING boolean_expr
order_by_clause  := ORDER BY order_item ("," order_item)*
order_item       := expr (ASC | DESC)?
limit_clause     := LIMIT NUMBER (OFFSET NUMBER)?

alias            := IDENT | quoted_ident
schema_name      := ident_like
table_name       := ident_like
column_name      := ident_like
ident_like       := IDENT | quoted_ident
quoted_ident     := "\"" IDENT "\""

expr             := or_expr
or_expr          := and_expr (OR and_expr)*
and_expr         := not_expr (AND not_expr)*
not_expr         := NOT? cmp_expr
cmp_expr         := add_expr ((=|<>|!=|<|<=|>|>=
                               |IS NULL|IS NOT NULL
                               |IN "(" expr_list ")"
                               |LIKE add_expr
                               |BETWEEN add_expr AND add_expr) add_expr)*
add_expr         := mul_expr ((+|-) mul_expr)*
mul_expr         := unary_expr ((*|/) unary_expr)*
unary_expr       := (+|-) unary_expr | primary
primary          := literal
                   | (qual_name ".*"?)
                   | "(" expr ")"
                   | function_call
qual_name        := (schema_name ".")? (table_name ".")? column_name
expr_list        := expr ("," expr)*

function_call    := ident_like "(" (DISTINCT? expr_list)? ")"

literal          := NUMBER | STRING | TRUE | FALSE | NULL

4. Abstract Syntax Tree (AST Structure)
Select {
  projections: [ExprAlias],
  from: [TableRef | Join],
  where?: Expr,
  groupBy?: Expr[],
  having?: Expr,
  orderBy?: OrderItem[],
  limit?: Limit
}

TableRef {
  name: QualifiedName,
  alias?: string
}

Join {
  type: 'Inner' | 'Left' | 'Right' | 'Full',
  left: FromItem,
  right: TableRef,
  on: Expr
}

Expr variants:
  - Literal
  - Column { qname }
  - Wildcard { qtable? }
  - Call { name, args, distinct? }
  - Unary { op, expr }
  - Binary { op, left, right }
  - Between { expr, low, high }
  - In { expr, list }
  - IsNull { negated? }

5. Example: SELECT g.* FROM "group" AS g;

⚠️ group is a reserved keyword, so it must be quoted in PostgreSQL.

SQL Input:

SELECT g.* FROM "group" AS g;


Token Stream:

SELECT  IDENT(g)  OP(.)  OP(*)  FROM  QUOTED_IDENT("group")  AS  IDENT(g)


Parsed AST:

{
  "type": "Select",
  "projections": [{ "type": "Wildcard", "table": "g" }],
  "from": [{ "type": "TableRef", "name": "group", "alias": "g" }]
}


Re-formatted Output:

SELECT g.*
FROM "group" AS g;

6. Parsing Notes (Implementation)

Dialect: PostgreSQL allows "Quoted Identifiers". Unquoted reserved words are invalid.

Parser type: Recursive-descent or Pratt parser (for expressions).

Error recovery: On unexpected tokens, skip until next clause (FROM, WHERE, GROUP, ORDER, LIMIT, ;).

Precedence:

1. Unary +/-
2. *, /
3. +, -
4. Comparison (=, <, >, LIKE, IN, IS)
5. NOT
6. AND
7. OR

7. Validation Rules

Wildcards (*, table.*) only valid in SELECT list.

Non-aggregated columns must appear in GROUP BY if aggregates are present.

Aliases shadow table names within their scope.

Quoted identifiers preserve case and must match exactly when referenced.

8. Example Workflow

Tokenize

SELECT → KEYWORD
g → IDENT
. → OP
* → OP
FROM → KEYWORD
"group" → QUOTED_IDENT
AS → KEYWORD
g → IDENT
; → DELIM


Parse → AST

Validate

Emit / execute / optimize