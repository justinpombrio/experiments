# Imports

Thoughts on how I would design a module/import system. A few very simple systems:

- `dependency` declares an external dependency, saying where to load from.
- `import` loads an internal or external dependency as a module. It converts
  files to source, and is only ever run once.
- `module` declares a module for namespacing.
- `use` declares an alias for an element of some module. Can this just be `let`?
  Can accesses be `foo.bar`, or do they have to be distinguished as `foo::bar`?

QUESTION: are modules in a different namespace than variables? Does the compiler
need to treat them very differently?

## Dependencies

Dependencies are declared in a package file:

    package CatCaptions;

    dependency std;
    dependency file "~/myRegex/" as myRegex;
    dependency git "github.com/zz-regex" as regex;

These mean:

    dependency std;
    -> importPaths.std = "$language/deps/std/"

    dependency file "~/myRegex/" as myRegex;
    -> importPaths.myRegex = "~/myRegex/"

    dependency git "github.com/zz-regex/" as regex
    -> download "github.com/zz-regex/" into "$package/target/deps/regex/"
    -> importPaths.regex = "$package/target/deps/regex/"

## Imports

To import an external dependency:

    import regex
    -> curmod.regex = cachedImport(importPaths.regex)

To import a sibiling file:

    import "foo.zz" 
    -> curmod.foo = cachedImport(curPath.parent().append("foo.zz"))

To import a submodule, from file "foo.zz":

    import "foo/fooImpl.zz"
    -> curmod.fooImpl = cachedImport(curPath.parent().append("foo").append("fooImpl.zz")

To import a helper module from above:

    import "../../helpers.zz"
    -> curmod.helpers = cachedImport(curPath.parent().parent().parent().append("helpers.zz")

    import "root/helpers.zz"
    -> curmod.helpers = cachedImport("$package/src/helpers.zz")

## Modules

Every file `foo.zz` must start with:

    module foo {
      ...
    }
    // Or the shorthand:
    module foo;

It may contain submodules within that:

    module foo;
    module fooImpl {
      ...
    }
    -> curmod.foo = new Module()
    -> curmod = curmod.foo
    ->   curmod.fooImpl = new Module()
    ->   curmod = curmod.fooImpl
    ->     ...
    ->   curmod = curmod.parent()
    -> curmod = curmod.parent()

## Uses

To use items in a module:

    use foo::bar::someFunc;
    -> curmod.someFunc = curmod.foo.bar.someFunc

    pub use foo::bar::someFunc;
    -> curmod.someFunc = curmod.foo.bar.someFunc
    -> curmod.pub.someFunc = curmod.foo.bar.someFunc

## Lints and Errors

- Lint: outer module name must match file name.
- Lint: can't import ".../foo/...", because the modules inside of "foo/" are
  private. If they are meant to be public, they will be exported by "foo.zz" and
  you should import them from there.
- Error: no imports from outside of "$package/src".
- Error: no cyclic imports.

## State

    -- Modified during package loading; shared after
    importPaths : Id -> Path
    -- Per file loader
    importChain : Vec<Path> -- detects cycles
    curmod      : Module

