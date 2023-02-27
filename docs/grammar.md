Formal grammar definition for Sloth.

```
program             → statement* ;
block               → "{" statement* "}"

statement           → exprStmt
                    | valStmt
                    | varStmt
                    | returnStmt
                    | printStmt
                    | functionStmt
                    | ifStmt
                    | forStmt ;

exprStmt            → expression ";" ;
valStmt             → "val" IDENTIFIER "=" expression ";" ;
varStmt             → "var" IDENTIFIER "=" expression ";" ;
returnStmt          → "return" expression ";" ;
printStmt           → "print" expression ";" ;

functionStmt        → "fn" IDENTIFIER "(" (IDENTIFIER ":" IDENTIFIER)* ")" block ;
ifStmt              → "if" expression block ;
forStmt             → "for" IDENTIFIER "in" expression ".." expression block ;

expression          → logical_or ;

logical_or          → logical_and ( "||" logical_and )* ;
logical_and         → equality ( "&&" equality )* ;
equality            → comparison ( ( "!=" | "==" ) comparison )* ;
comparison          → bitwise_shift ( ( "<" | ">" | "<=" | ">=" ) bitwise_shift )* ;
bitwise_shifting    → additive ( ( "<<" | ">>" ) additive )* ;
additive            → multiplicative ( ( "+" | "-" ) multiplicative )* ;
multiplicative      → unary ( ( "*" | "/" | "%" ) unary )* ;
unary               → ( "!" | "+" | "-" ) unary | call ;

call                → primary ( "(" arguments? ")" )* ;
primary             → "true" | "false" | NUMBER | STRING | IDENTIFIER | "(" expression ")" ;
```
