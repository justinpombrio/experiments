# vim: set syntax=tardigrade :

// The Ultimate Conditional Syntax:
// https://dl.acm.org/doi/pdf/10.1145/3689746

// Matklad's desiderata:
// 1. unified if/match
// 2. dependent patterns
// 3. first class multi-way cond (a-la Kotlin when), for when you want to check the related conditions
// 4. concise ternary
// 5. no double-indentation when matching

// 1. Unified if/match

    if x > 0 {
        ...
    } else {
        ...
    }
    
    if xs.pop()
    is Some(x) { ... }
    is None { ... }

// 2. dependent patterns

    // while other_iter.peek().is_some() && *other_iter.peek().unwrap() <= x { ... }
    while other_iter.peek() is Some(elem) and elem <= x { ... }
    
    // if let KeyCode::Char(ch) = self.code and self.modifiers.shift { ... }
    if self.code is KeyCode::Char(ch) and self.modifiers.shift { ... }
    
    // if self.cursor_loc.and_then(|loc| loc.node(self.storage)) == Some(self.node) { ... }
    if self.cursor_loc is Some(loc) and loc.node(self.storage) == Some(self.node) { ... }

// 3. first class multi-way cond (a-la Kotlin when), for when you want to check the related conditions

    let sign = if
    case x > 0 { 1 }
    case x < 0 { -1 }
    else { 0 }

// 4. concise ternary

    let sign = if x > 0 { 1 } else { -1 }

// 5. no double-indentation when matching

    if xs.pop()
    is Some(x) {
        ...
    } is None {
        ...
    }

// Misc

    if xs.pop()
    is Some(x) and ys.pop()
        is Some(y) { Some(merge(x, y)) }
        else { Some(x) }
    else None

    // if let Some(construct_id) = self.constructs.id(name) {
    //     ...
    // } else if let Some(child_sort) = self.sorts.get(name) {
    //     ...
    // } else { ... }
    if self.constructs.id(name) is Some(construct_id) {
        ...
    } else if self.sorts.get(name) is Some(child_sort) {
        ...
    } else { ... }

/////////////////////////////////////
// Now with significant whitespace //
/////////////////////////////////////

// 1. Unified if/match

    if x > 0 then
        ...
    else
        ...
    
    if xs.pop()
    is Some(x) then ...
    is None then ...

// 2. dependent patterns

    // while other_iter.peek().is_some() && *other_iter.peek().unwrap() <= x { ... }
    while other_iter.peek() is Some(elem) and elem <= x
    
    // if let KeyCode::Char(ch) = self.code and self.modifiers.shift { ... }
    if self.code is KeyCode::Char(ch) and self.modifiers.shift
    
    // if self.cursor_loc.and_then(|loc| loc.node(self.storage)) == Some(self.node) { ... }
    if self.cursor_loc is Some(loc) and loc.node(self.storage) == Some(self.node)

// 3. first class multi-way cond (a-la Kotlin when), for when you want to check the related conditions

    let sign = if
        case x > 0 then 1
        case x < 0 then -1
        else 0

    if
    case x is Some(z) then f(z)
    case y is Ok(z) then f(z)
    else f(0)

    if
    case x
        is Ok(z) then f(z)
        is Err(z) then f(z)
    case y is Ok(z) then f(z)
    else f(0)

// 4. concise ternary

    let sign = if x > 0 then 1 else -1

// 5. no double-indentation when matching

    if xs.pop()
    is Some(x) then
        ...
    is None then
        ...

/////////////////////
// General Grammar //
/////////////////////

IF ::= if SPLIT

SPLIT ::=
  | COND THEN (else EXPR)?
  | (case COND THEN)+ (else EXPR)?
  | EXPR (is PATTERN THEN)+ (else EXPR)?

COND ::= 
  | EXPR
  | EXPR is PATTERN

THEN ::=
  | then EXPR
  | and SPLIT

///////////////////
// Open Question //
///////////////////

How to handle `or`?
