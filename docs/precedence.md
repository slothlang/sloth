Operating precedence in sloth from highest to lowest.

| Name           | Operators | Associates |
| -------------- | --------- | ---------- |
| parentheses    | ()        | Left       |
| member access  | . ! !! ?. | Left       |
| defaulting     | ?:        | Right      |
| unary          | ! + -     | Right      |
| multiplicative | \* / %    | Left       |
| additive       | + -       | Left       |
| bitwise shift  | << >>     | Left       |
| comparison     | < > <= >= | Left       |
| equality       | == !=     | Left       |
| bitwise        | & ^ \|    | Left       |
| logical and    | &&        | Left       |
| logical or     | \|\|      | Left       |
