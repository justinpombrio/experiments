use crate::{ChoiceP, Parser, SeqP};
use std::marker::PhantomData;

/*========================================*/
/*          Parsers: Choice               */
/*========================================*/

pub fn choice<T, C: ChoiceTuple<T>>(name: &str, tuple: C) -> impl Parser<T> + Clone {
    tuple.make_choice(name.to_owned())
}

pub trait ChoiceTuple<T> {
    fn make_choice(self, name: String) -> impl Parser<T> + Clone;
}

impl<T, P0, P1> ChoiceTuple<T> for (P0, P1)
where
    P0: Parser<T> + Clone,
    P1: Parser<T> + Clone,
{
    fn make_choice(self, name: String) -> impl Parser<T> + Clone {
        ChoiceP {
            name,
            parser_0: self.0,
            parser_1: self.1,
            phantom: PhantomData,
        }
    }
}

impl<T, P0, P1, P2> ChoiceTuple<T> for (P0, P1, P2)
where
    P0: Parser<T> + Clone,
    P1: Parser<T> + Clone,
    P2: Parser<T> + Clone,
{
    fn make_choice(self, name: String) -> impl Parser<T> + Clone {
        let parser = self.2;
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<T, P0, P1, P2, P3> ChoiceTuple<T> for (P0, P1, P2, P3)
where
    P0: Parser<T> + Clone,
    P1: Parser<T> + Clone,
    P2: Parser<T> + Clone,
    P3: Parser<T> + Clone,
{
    fn make_choice(self, name: String) -> impl Parser<T> + Clone {
        let parser = self.3;
        let parser = (self.2, parser).make_choice(name.clone());
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<T, P0, P1, P2, P3, P4> ChoiceTuple<T> for (P0, P1, P2, P3, P4)
where
    P0: Parser<T> + Clone,
    P1: Parser<T> + Clone,
    P2: Parser<T> + Clone,
    P3: Parser<T> + Clone,
    P4: Parser<T> + Clone,
{
    fn make_choice(self, name: String) -> impl Parser<T> + Clone {
        let parser = self.4;
        let parser = (self.3, parser).make_choice(name.clone());
        let parser = (self.2, parser).make_choice(name.clone());
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<T, P0, P1, P2, P3, P4, P5> ChoiceTuple<T> for (P0, P1, P2, P3, P4, P5)
where
    P0: Parser<T> + Clone,
    P1: Parser<T> + Clone,
    P2: Parser<T> + Clone,
    P3: Parser<T> + Clone,
    P4: Parser<T> + Clone,
    P5: Parser<T> + Clone,
{
    fn make_choice(self, name: String) -> impl Parser<T> + Clone {
        let parser = self.5;
        let parser = (self.4, parser).make_choice(name.clone());
        let parser = (self.3, parser).make_choice(name.clone());
        let parser = (self.2, parser).make_choice(name.clone());
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<T, P0, P1, P2, P3, P4, P5, P6> ChoiceTuple<T> for (P0, P1, P2, P3, P4, P5, P6)
where
    P0: Parser<T> + Clone,
    P1: Parser<T> + Clone,
    P2: Parser<T> + Clone,
    P3: Parser<T> + Clone,
    P4: Parser<T> + Clone,
    P5: Parser<T> + Clone,
    P6: Parser<T> + Clone,
{
    fn make_choice(self, name: String) -> impl Parser<T> + Clone {
        let parser = self.6;
        let parser = (self.5, parser).make_choice(name.clone());
        let parser = (self.4, parser).make_choice(name.clone());
        let parser = (self.3, parser).make_choice(name.clone());
        let parser = (self.2, parser).make_choice(name.clone());
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<T, P0, P1, P2, P3, P4, P5, P6, P7> ChoiceTuple<T> for (P0, P1, P2, P3, P4, P5, P6, P7)
where
    P0: Parser<T> + Clone,
    P1: Parser<T> + Clone,
    P2: Parser<T> + Clone,
    P3: Parser<T> + Clone,
    P4: Parser<T> + Clone,
    P5: Parser<T> + Clone,
    P6: Parser<T> + Clone,
    P7: Parser<T> + Clone,
{
    fn make_choice(self, name: String) -> impl Parser<T> + Clone {
        let parser = self.7;
        let parser = (self.6, parser).make_choice(name.clone());
        let parser = (self.5, parser).make_choice(name.clone());
        let parser = (self.4, parser).make_choice(name.clone());
        let parser = (self.3, parser).make_choice(name.clone());
        let parser = (self.2, parser).make_choice(name.clone());
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

/*========================================*/
/*          Parsers: Seq                  */
/*========================================*/

pub fn tuple<T, S: SeqTuple<T>>(tuple: S) -> impl Parser<T> + Clone {
    tuple.make_seq()
}

pub trait SeqTuple<T> {
    fn make_seq(self) -> impl Parser<T> + Clone;
}

impl<T0, T1, P0, P1> SeqTuple<(T0, T1)> for (P0, P1)
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
{
    fn make_seq(self) -> impl Parser<(T0, T1)> + Clone {
        SeqP(self.0, self.1, PhantomData)
    }
}

impl<T0, T1, T2, P0, P1, P2> SeqTuple<(T0, T1, T2)> for (P0, P1, P2)
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    P2: Parser<T2> + Clone,
{
    fn make_seq(self) -> impl Parser<(T0, T1, T2)> + Clone {
        let parser = self.2;
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, c))| (a, b, c))
    }
}

impl<T0, T1, T2, T3, P0, P1, P2, P3> SeqTuple<(T0, T1, T2, T3)> for (P0, P1, P2, P3)
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    P2: Parser<T2> + Clone,
    P3: Parser<T3> + Clone,
{
    fn make_seq(self) -> impl Parser<(T0, T1, T2, T3)> + Clone {
        let parser = self.3;
        let parser = (self.2, parser).make_seq();
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, (c, d)))| (a, b, c, d))
    }
}

impl<T0, T1, T2, T3, T4, P0, P1, P2, P3, P4> SeqTuple<(T0, T1, T2, T3, T4)> for (P0, P1, P2, P3, P4)
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    P2: Parser<T2> + Clone,
    P3: Parser<T3> + Clone,
    P4: Parser<T4> + Clone,
{
    fn make_seq(self) -> impl Parser<(T0, T1, T2, T3, T4)> + Clone {
        let parser = self.4;
        let parser = (self.3, parser).make_seq();
        let parser = (self.2, parser).make_seq();
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, (c, (d, e))))| (a, b, c, d, e))
    }
}

impl<T0, T1, T2, T3, T4, T5, P0, P1, P2, P3, P4, P5> SeqTuple<(T0, T1, T2, T3, T4, T5)>
    for (P0, P1, P2, P3, P4, P5)
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    P2: Parser<T2> + Clone,
    P3: Parser<T3> + Clone,
    P4: Parser<T4> + Clone,
    P5: Parser<T5> + Clone,
{
    fn make_seq(self) -> impl Parser<(T0, T1, T2, T3, T4, T5)> + Clone {
        let parser = self.5;
        let parser = (self.4, parser).make_seq();
        let parser = (self.3, parser).make_seq();
        let parser = (self.2, parser).make_seq();
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, (c, (d, (e, f)))))| (a, b, c, d, e, f))
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, P0, P1, P2, P3, P4, P5, P6> SeqTuple<(T0, T1, T2, T3, T4, T5, T6)>
    for (P0, P1, P2, P3, P4, P5, P6)
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    P2: Parser<T2> + Clone,
    P3: Parser<T3> + Clone,
    P4: Parser<T4> + Clone,
    P5: Parser<T5> + Clone,
    P6: Parser<T6> + Clone,
{
    fn make_seq(self) -> impl Parser<(T0, T1, T2, T3, T4, T5, T6)> + Clone {
        let parser = self.6;
        let parser = (self.5, parser).make_seq();
        let parser = (self.4, parser).make_seq();
        let parser = (self.3, parser).make_seq();
        let parser = (self.2, parser).make_seq();
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, (c, (d, (e, (f, g))))))| (a, b, c, d, e, f, g))
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7, P0, P1, P2, P3, P4, P5, P6, P7>
    SeqTuple<(T0, T1, T2, T3, T4, T5, T6, T7)> for (P0, P1, P2, P3, P4, P5, P6, P7)
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    P2: Parser<T2> + Clone,
    P3: Parser<T3> + Clone,
    P4: Parser<T4> + Clone,
    P5: Parser<T5> + Clone,
    P6: Parser<T6> + Clone,
    P7: Parser<T7> + Clone,
{
    fn make_seq(self) -> impl Parser<(T0, T1, T2, T3, T4, T5, T6, T7)> + Clone {
        let parser = self.7;
        let parser = (self.6, parser).make_seq();
        let parser = (self.5, parser).make_seq();
        let parser = (self.4, parser).make_seq();
        let parser = (self.3, parser).make_seq();
        let parser = (self.2, parser).make_seq();
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, (c, (d, (e, (f, (g, h)))))))| (a, b, c, d, e, f, g, h))
    }
}
