:syntax keyword Statement while fn foreign fn var if else return as in
:syntax keyword Type Int String Float Void Bool
:syntax match Number '[1234567890]'
:syntax match Operator '[+ \- \* \/ <= == >= &&]'
:syntax region String start=+"+ skip=+\\+ end=+"+
:syntax match paren "("
:syntax match Function "\w\+\s*(" contains=paren
:syntax match Comment "#.*"
