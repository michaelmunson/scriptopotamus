---
title: Macros
description: Compile-time macros defined with @name and expanded at the call.
---

A macro is a named block that the compiler pastes in place of each call. The paste happens after imports are resolved and before `@inherit` and type checking. Macros produce no bash function of their own.

## Defining and calling

A `@` name followed by parentheses, with a deeper indented body, is a definition. A `@` name without a body is a call.

```
@say_hi()
  -> "hi from macro"

@say_hi
```

The call disappears. The `->` statement is compiled where the call stood. Calls may appear inside functions, commands, `if` bodies, `case` arms, and other macros. A macro may call another macro. A call to an unknown macro is a compile error.

A definition whose parentheses contain words stores those words and does not use them. Calls may also be written with words (`@say_hi there`). The words are parsed and discarded. The body is inserted exactly as it was written. When a snippet needs arguments, write a [function](/functions).

## Calls inside bash

A bash line whose first non-whitespace text is `@name` is expanded when that macro exists. `->` statements in the macro body become `echo` lines. Nested bash is copied. Other statement kinds inside a macro expanded from a bash line become a comment, so keep those macros to `->` and bash if you call them from a bash line.

```
@say_hi()
  -> "hi from macro"

risky || {
  @say_hi
}
```

The braces are bash. The macro call is the bash line inside them, and it expands to `echo`.

## Built-in @ names

These are compiler forms, not macros you can define:

| Form | Role |
| --- | --- |
| `@import` | Compile-time inclusion. [Imports](/imports). |
| `@info` | File metadata. [Metadata](/metadata). |
| `@inherit` | Parameter copying. [Inherit](/inherit). |
| `@usage` | Print usage. [Usage text](/usage). |

`@info` is only legal at the top of a file. Defining a macro named `import`, `info`, `inherit`, or `usage` will collide with these forms; pick another name.

## Order of expansion

1. `@info` is read and removed.
2. `@import` is loaded, recursively, and removed.
3. Macro definitions are collected from the whole tree, then calls are replaced by bodies.
4. `@inherit` is resolved.
5. The program is type-checked and emitted.

A macro defined in an imported file is available to the importer, because the definition is pasted in before macro expansion.
