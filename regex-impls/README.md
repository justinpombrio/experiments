# regex-impls

Regex implementations in Rust, for fun.

- `deriv_regex` uses regular expression derivatives and smart constructors, as described in
["Smart constructors are smarter than you think"](http://www.weaselhat.com/2020/05/07/smart-constructors-are-smarter-than-you-think/).
- `combinator_regex` defined a combinator interface for regexes. The interface has the regex expose
  a state, which you can advance by a character, ask whether it accepts at this moment, and such.

For each, I implemented the regex `^(0|[1-9][0-9]*)(\\.[0-9]*)?$`, tested it on a couple length 50
strings, and compared to Burnt Sushi's `regex` crate, which is probably as fast as you can get. The
timing on my desktop is:

                *-------------*------------------*-------------*
                | regex crate | combinator_regex | deriv_regex |
    *-----------*-------------*------------------*-------------*
    | time      | 227 ns      | 666 ns           | 5000 ns     |
    *-----------*-------------*------------------*-------------*
    | time/char | 2.7 ns      | 6.6 ns           | 50 ns       |
    *-----------*-------------*------------------*-------------*
    | memory    | 4000 bytes  | 64 bytes         | grows :-/   |
    *-----------*-------------*------------------*-------------*
    (On my laptop)
