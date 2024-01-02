use crate::{ChoiceP, Parser, SeqP};

/*========================================*/
/*          Parsers: Choice               */
/*========================================*/

pub fn choice_2<O>(
    name: &str,
    parser_0: impl Parser<Output = O> + Clone,
    parser_1: impl Parser<Output = O> + Clone,
) -> impl Parser<Output = O> + Clone {
    ChoiceP {
        name: name.to_owned(),
        parser_0,
        parser_1,
    }
}

pub fn choice_3<O>(
    name: &str,
    parser_0: impl Parser<Output = O> + Clone,
    parser_1: impl Parser<Output = O> + Clone,
    parser_2: impl Parser<Output = O> + Clone,
) -> impl Parser<Output = O> + Clone {
    let parser = parser_2;
    let parser = choice_2(name, parser_1, parser);
    let parser = choice_2(name, parser_0, parser);
    parser
}

pub fn choice_4<O>(
    name: &str,
    parser_0: impl Parser<Output = O> + Clone,
    parser_1: impl Parser<Output = O> + Clone,
    parser_2: impl Parser<Output = O> + Clone,
    parser_3: impl Parser<Output = O> + Clone,
) -> impl Parser<Output = O> + Clone {
    let parser = parser_3;
    let parser = choice_2(name, parser_2, parser);
    let parser = choice_2(name, parser_1, parser);
    let parser = choice_2(name, parser_0, parser);
    parser
}

pub fn choice_5<O>(
    name: &str,
    parser_0: impl Parser<Output = O> + Clone,
    parser_1: impl Parser<Output = O> + Clone,
    parser_2: impl Parser<Output = O> + Clone,
    parser_3: impl Parser<Output = O> + Clone,
    parser_4: impl Parser<Output = O> + Clone,
) -> impl Parser<Output = O> + Clone {
    let parser = parser_4;
    let parser = choice_2(name, parser_3, parser);
    let parser = choice_2(name, parser_2, parser);
    let parser = choice_2(name, parser_1, parser);
    let parser = choice_2(name, parser_0, parser);
    parser
}

pub fn choice_6<O>(
    name: &str,
    parser_0: impl Parser<Output = O> + Clone,
    parser_1: impl Parser<Output = O> + Clone,
    parser_2: impl Parser<Output = O> + Clone,
    parser_3: impl Parser<Output = O> + Clone,
    parser_4: impl Parser<Output = O> + Clone,
    parser_5: impl Parser<Output = O> + Clone,
) -> impl Parser<Output = O> + Clone {
    let parser = parser_5;
    let parser = choice_2(name, parser_4, parser);
    let parser = choice_2(name, parser_3, parser);
    let parser = choice_2(name, parser_2, parser);
    let parser = choice_2(name, parser_1, parser);
    let parser = choice_2(name, parser_0, parser);
    parser
}

pub fn choice_7<O>(
    name: &str,
    parser_0: impl Parser<Output = O> + Clone,
    parser_1: impl Parser<Output = O> + Clone,
    parser_2: impl Parser<Output = O> + Clone,
    parser_3: impl Parser<Output = O> + Clone,
    parser_4: impl Parser<Output = O> + Clone,
    parser_5: impl Parser<Output = O> + Clone,
    parser_6: impl Parser<Output = O> + Clone,
) -> impl Parser<Output = O> + Clone {
    let parser = parser_6;
    let parser = choice_2(name, parser_5, parser);
    let parser = choice_2(name, parser_4, parser);
    let parser = choice_2(name, parser_3, parser);
    let parser = choice_2(name, parser_2, parser);
    let parser = choice_2(name, parser_1, parser);
    let parser = choice_2(name, parser_0, parser);
    parser
}

pub fn choice_8<O>(
    name: &str,
    parser_0: impl Parser<Output = O> + Clone,
    parser_1: impl Parser<Output = O> + Clone,
    parser_2: impl Parser<Output = O> + Clone,
    parser_3: impl Parser<Output = O> + Clone,
    parser_4: impl Parser<Output = O> + Clone,
    parser_5: impl Parser<Output = O> + Clone,
    parser_6: impl Parser<Output = O> + Clone,
    parser_7: impl Parser<Output = O> + Clone,
) -> impl Parser<Output = O> + Clone {
    let parser = parser_7;
    let parser = choice_2(name, parser_6, parser);
    let parser = choice_2(name, parser_5, parser);
    let parser = choice_2(name, parser_4, parser);
    let parser = choice_2(name, parser_3, parser);
    let parser = choice_2(name, parser_2, parser);
    let parser = choice_2(name, parser_1, parser);
    let parser = choice_2(name, parser_0, parser);
    parser
}

/*========================================*/
/*          Parsers: Seq                  */
/*========================================*/

pub fn seq_2<P0: Parser + Clone, P1: Parser + Clone>(
    parser_0: P0,
    parser_1: P1,
) -> impl Parser<Output = (P0::Output, P1::Output)> + Clone
where
    P0::Output: Clone,
    P1::Output: Clone,
{
    SeqP(parser_0, parser_1)
}

pub fn seq_3<P0: Parser + Clone, P1: Parser + Clone, P2: Parser + Clone>(
    parser_0: P0,
    parser_1: P1,
    parser_2: P2,
) -> impl Parser<Output = (P0::Output, P1::Output, P2::Output)> + Clone
where
    P0::Output: Clone,
    P1::Output: Clone,
    P2::Output: Clone,
{
    seq_2(parser_0, seq_2(parser_1, parser_2)).map(|(a, (b, c))| (a, b, c))
}

pub fn seq_4<P0: Parser + Clone, P1: Parser + Clone, P2: Parser + Clone, P3: Parser + Clone>(
    parser_0: P0,
    parser_1: P1,
    parser_2: P2,
    parser_3: P3,
) -> impl Parser<Output = (P0::Output, P1::Output, P2::Output, P3::Output)> + Clone
where
    P0::Output: Clone,
    P1::Output: Clone,
    P2::Output: Clone,
    P3::Output: Clone,
{
    seq_2(parser_0, seq_2(parser_1, seq_2(parser_2, parser_3))).map(|(a, (b, (c, d)))| (a, b, c, d))
}

pub fn seq_5<
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
    P4: Parser + Clone,
>(
    parser_0: P0,
    parser_1: P1,
    parser_2: P2,
    parser_3: P3,
    parser_4: P4,
) -> impl Parser<Output = (P0::Output, P1::Output, P2::Output, P3::Output, P4::Output)> + Clone
where
    P0::Output: Clone,
    P1::Output: Clone,
    P2::Output: Clone,
    P3::Output: Clone,
    P4::Output: Clone,
{
    seq_2(
        parser_0,
        seq_2(parser_1, seq_2(parser_2, seq_2(parser_3, parser_4))),
    )
    .map(|(a, (b, (c, (d, e))))| (a, b, c, d, e))
}

pub fn seq_6<
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
    P4: Parser + Clone,
    P5: Parser + Clone,
>(
    parser_0: P0,
    parser_1: P1,
    parser_2: P2,
    parser_3: P3,
    parser_4: P4,
    parser_5: P5,
) -> impl Parser<
    Output = (
        P0::Output,
        P1::Output,
        P2::Output,
        P3::Output,
        P4::Output,
        P5::Output,
    ),
> + Clone
where
    P0::Output: Clone,
    P1::Output: Clone,
    P2::Output: Clone,
    P3::Output: Clone,
    P4::Output: Clone,
    P5::Output: Clone,
{
    seq_2(
        parser_0,
        seq_2(
            parser_1,
            seq_2(parser_2, seq_2(parser_3, seq_2(parser_4, parser_5))),
        ),
    )
    .map(|(a, (b, (c, (d, (e, f)))))| (a, b, c, d, e, f))
}

pub fn seq_7<
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
    P4: Parser + Clone,
    P5: Parser + Clone,
    P6: Parser + Clone,
>(
    parser_0: P0,
    parser_1: P1,
    parser_2: P2,
    parser_3: P3,
    parser_4: P4,
    parser_5: P5,
    parser_6: P6,
) -> impl Parser<
    Output = (
        P0::Output,
        P1::Output,
        P2::Output,
        P3::Output,
        P4::Output,
        P5::Output,
        P6::Output,
    ),
> + Clone
where
    P0::Output: Clone,
    P1::Output: Clone,
    P2::Output: Clone,
    P3::Output: Clone,
    P4::Output: Clone,
    P5::Output: Clone,
    P6::Output: Clone,
{
    seq_2(
        parser_0,
        seq_2(
            parser_1,
            seq_2(
                parser_2,
                seq_2(parser_3, seq_2(parser_4, seq_2(parser_5, parser_6))),
            ),
        ),
    )
    .map(|(a, (b, (c, (d, (e, (f, g))))))| (a, b, c, d, e, f, g))
}

pub fn seq_8<
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
    P4: Parser + Clone,
    P5: Parser + Clone,
    P6: Parser + Clone,
    P7: Parser + Clone,
>(
    parser_0: P0,
    parser_1: P1,
    parser_2: P2,
    parser_3: P3,
    parser_4: P4,
    parser_5: P5,
    parser_6: P6,
    parser_7: P7,
) -> impl Parser<
    Output = (
        P0::Output,
        P1::Output,
        P2::Output,
        P3::Output,
        P4::Output,
        P5::Output,
        P6::Output,
        P7::Output,
    ),
> + Clone
where
    P0::Output: Clone,
    P1::Output: Clone,
    P2::Output: Clone,
    P3::Output: Clone,
    P4::Output: Clone,
    P5::Output: Clone,
    P6::Output: Clone,
    P7::Output: Clone,
{
    seq_2(
        parser_0,
        seq_2(
            parser_1,
            seq_2(
                parser_2,
                seq_2(
                    parser_3,
                    seq_2(parser_4, seq_2(parser_5, seq_2(parser_6, parser_7))),
                ),
            ),
        ),
    )
    .map(|(a, (b, (c, (d, (e, (f, (g, h)))))))| (a, b, c, d, e, f, g, h))
}
