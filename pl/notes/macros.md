# Macros

Here's my thoughts on how typed macros could work.

At a high level, there is language support for typed quoted expressions, and a
macro gives code run to run at compile time that constructs an expression. This
requires existential types to work.

## Supporting Methods

    impl Compiler {
        // Type check and compile an expression.
        // ExprSrc is a node in the parse tree of nonterminal Expr.
        fn compileExpr(self, src: ExprSrc) -> ex T. Expr<T> throws TypeError;
    
        // Type check and compile a pattern->template pair.
        // For example, in `lam (x, y). x * y`:
        //   pattern = "(x, y)"
        //   template = "x * y"
        //   T = (int, int)
        //   U = int
        fn compilePattern<T>(self, pattern: PattSrc, template: ExprSrc)
        -> ex U. Expr<T> -> Expr<U> throws TypeError;
    }

`assert CONDITION;` means to check that type constraint CONDITION holds, and to
throw a TypeError if it does not.

If a TypeError is thrown in the `semantics` portion of a `macro` (_not_
including inside calls to `compileExpr` and such), the error will mention the
name of the macro, such as "addition expression".

## Macro Examples

    macro "addition expression"
        syntax `$a:ExprSrc + $b:ExprSrc`
        semantics
            fn compile(c: Compiler, a: ExprSrc, b: ExprSrc)
            -> ex R. Expr<R> throws TypeError
            {
                let ex T. a2 = c.compileExpr(a)?;
                let ex U. b2 = c.compileExpr(b)?;
                assert Add<T, U>?;
                ex Add<T, U>::Output. `$a2.add($b2)`
            }
        end
    end
    
    macro "let expression"
        syntax `let $p:PattSrc = $a:ExprSrc in $b:ExprSrc`
        semantics
            fn compile(c: Compiler, p: PattSrc, a: ExprSrc, b: ExprSrc)
            -> ex R. Expr<R> throws TypeError
            {
                let ex T. a = compileExpr(a)?;
                let ex U. f = compilePattern<T>(p, b)?;
                // a: Expr<T>
                // f: Expr<T> -> Expr<U>
                // f a: Expr<U>
                ex U. f a
            }
        end
    end

    macro "if expression"
        syntax `if $b: ExprSrc then $x: ExprSrc else $y: ExprSrc`
        semantics
            fn compile(c: Compiler, b: ExprSrc, x: ExprSrc, y: ExprSrc)
            -> ex R. Expr<R> throws TypeError
            {
                let b = compileExprOfType<Bool>(b)?;
                let ex X. x = compileExpr(x)?;
                let y = compileExprOfType<X>(y)?;
                ex X. `conditional($b, $x, $y)`
            }
        end
    end

