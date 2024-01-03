use crate::{ChoiceP, Parser, SeqP};

/*========================================*/
/*          Parsers: Choice               */
/*========================================*/

pub fn choice<T: ChoiceTuple>(name: &str, tuple: T) -> impl Parser<Output = T::Output> + Clone {
    tuple.make_choice(name.to_owned())
}

pub trait ChoiceTuple {
    type Output;

    fn make_choice(self, name: String) -> impl Parser<Output = Self::Output> + Clone;
}

impl<O, P0, P1> ChoiceTuple for (P0, P1)
where
    P0: Parser<Output = O> + Clone,
    P1: Parser<Output = O> + Clone,
{
    type Output = O;

    fn make_choice(self, name: String) -> impl Parser<Output = Self::Output> + Clone {
        ChoiceP {
            name,
            parser_0: self.0,
            parser_1: self.1,
        }
    }
}

impl<O, P0, P1, P2> ChoiceTuple for (P0, P1, P2)
where
    P0: Parser<Output = O> + Clone,
    P1: Parser<Output = O> + Clone,
    P2: Parser<Output = O> + Clone,
{
    type Output = O;

    fn make_choice(self, name: String) -> impl Parser<Output = Self::Output> + Clone {
        let parser = self.2;
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<O, P0, P1, P2, P3> ChoiceTuple for (P0, P1, P2, P3)
where
    P0: Parser<Output = O> + Clone,
    P1: Parser<Output = O> + Clone,
    P2: Parser<Output = O> + Clone,
    P3: Parser<Output = O> + Clone,
{
    type Output = O;

    fn make_choice(self, name: String) -> impl Parser<Output = Self::Output> + Clone {
        let parser = self.3;
        let parser = (self.2, parser).make_choice(name.clone());
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<O, P0, P1, P2, P3, P4> ChoiceTuple for (P0, P1, P2, P3, P4)
where
    P0: Parser<Output = O> + Clone,
    P1: Parser<Output = O> + Clone,
    P2: Parser<Output = O> + Clone,
    P3: Parser<Output = O> + Clone,
    P4: Parser<Output = O> + Clone,
{
    type Output = O;

    fn make_choice(self, name: String) -> impl Parser<Output = Self::Output> + Clone {
        let parser = self.4;
        let parser = (self.3, parser).make_choice(name.clone());
        let parser = (self.2, parser).make_choice(name.clone());
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<O, P0, P1, P2, P3, P4, P5> ChoiceTuple for (P0, P1, P2, P3, P4, P5)
where
    P0: Parser<Output = O> + Clone,
    P1: Parser<Output = O> + Clone,
    P2: Parser<Output = O> + Clone,
    P3: Parser<Output = O> + Clone,
    P4: Parser<Output = O> + Clone,
    P5: Parser<Output = O> + Clone,
{
    type Output = O;

    fn make_choice(self, name: String) -> impl Parser<Output = Self::Output> + Clone {
        let parser = self.5;
        let parser = (self.4, parser).make_choice(name.clone());
        let parser = (self.3, parser).make_choice(name.clone());
        let parser = (self.2, parser).make_choice(name.clone());
        let parser = (self.1, parser).make_choice(name.clone());
        let parser = (self.0, parser).make_choice(name);
        parser
    }
}

impl<O, P0, P1, P2, P3, P4, P5, P6> ChoiceTuple for (P0, P1, P2, P3, P4, P5, P6)
where
    P0: Parser<Output = O> + Clone,
    P1: Parser<Output = O> + Clone,
    P2: Parser<Output = O> + Clone,
    P3: Parser<Output = O> + Clone,
    P4: Parser<Output = O> + Clone,
    P5: Parser<Output = O> + Clone,
    P6: Parser<Output = O> + Clone,
{
    type Output = O;

    fn make_choice(self, name: String) -> impl Parser<Output = Self::Output> + Clone {
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

impl<O, P0, P1, P2, P3, P4, P5, P6, P7> ChoiceTuple for (P0, P1, P2, P3, P4, P5, P6, P7)
where
    P0: Parser<Output = O> + Clone,
    P1: Parser<Output = O> + Clone,
    P2: Parser<Output = O> + Clone,
    P3: Parser<Output = O> + Clone,
    P4: Parser<Output = O> + Clone,
    P5: Parser<Output = O> + Clone,
    P6: Parser<Output = O> + Clone,
    P7: Parser<Output = O> + Clone,
{
    type Output = O;

    fn make_choice(self, name: String) -> impl Parser<Output = Self::Output> + Clone {
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

pub fn seq<T: SeqTuple>(tuple: T) -> impl Parser<Output = T::Output> + Clone {
    tuple.make_seq()
}

pub trait SeqTuple {
    type Output;

    fn make_seq(self) -> impl Parser<Output = Self::Output> + Clone;
}

impl<P0, P1> SeqTuple for (P0, P1)
where
    P0: Parser + Clone,
    P1: Parser + Clone,
{
    type Output = (P0::Output, P1::Output);

    fn make_seq(self) -> impl Parser<Output = Self::Output> + Clone {
        SeqP(self.0, self.1)
    }
}

impl<P0, P1, P2> SeqTuple for (P0, P1, P2)
where
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
{
    type Output = (P0::Output, P1::Output, P2::Output);

    fn make_seq(self) -> impl Parser<Output = Self::Output> + Clone {
        let parser = self.2;
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, c))| (a, b, c))
    }
}

impl<P0, P1, P2, P3> SeqTuple for (P0, P1, P2, P3)
where
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
{
    type Output = (P0::Output, P1::Output, P2::Output, P3::Output);

    fn make_seq(self) -> impl Parser<Output = Self::Output> + Clone {
        let parser = self.3;
        let parser = (self.2, parser).make_seq();
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, (c, d)))| (a, b, c, d))
    }
}

impl<P0, P1, P2, P3, P4> SeqTuple for (P0, P1, P2, P3, P4)
where
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
    P4: Parser + Clone,
{
    type Output = (P0::Output, P1::Output, P2::Output, P3::Output, P4::Output);

    fn make_seq(self) -> impl Parser<Output = Self::Output> + Clone {
        let parser = self.4;
        let parser = (self.3, parser).make_seq();
        let parser = (self.2, parser).make_seq();
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, (c, (d, e))))| (a, b, c, d, e))
    }
}

impl<P0, P1, P2, P3, P4, P5> SeqTuple for (P0, P1, P2, P3, P4, P5)
where
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
    P4: Parser + Clone,
    P5: Parser + Clone,
{
    type Output = (
        P0::Output,
        P1::Output,
        P2::Output,
        P3::Output,
        P4::Output,
        P5::Output,
    );

    fn make_seq(self) -> impl Parser<Output = Self::Output> + Clone {
        let parser = self.5;
        let parser = (self.4, parser).make_seq();
        let parser = (self.3, parser).make_seq();
        let parser = (self.2, parser).make_seq();
        let parser = (self.1, parser).make_seq();
        let parser = (self.0, parser).make_seq();
        parser.map(|(a, (b, (c, (d, (e, f)))))| (a, b, c, d, e, f))
    }
}

impl<P0, P1, P2, P3, P4, P5, P6> SeqTuple for (P0, P1, P2, P3, P4, P5, P6)
where
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
    P4: Parser + Clone,
    P5: Parser + Clone,
    P6: Parser + Clone,
{
    type Output = (
        P0::Output,
        P1::Output,
        P2::Output,
        P3::Output,
        P4::Output,
        P5::Output,
        P6::Output,
    );

    fn make_seq(self) -> impl Parser<Output = Self::Output> + Clone {
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

impl<P0, P1, P2, P3, P4, P5, P6, P7> SeqTuple for (P0, P1, P2, P3, P4, P5, P6, P7)
where
    P0: Parser + Clone,
    P1: Parser + Clone,
    P2: Parser + Clone,
    P3: Parser + Clone,
    P4: Parser + Clone,
    P5: Parser + Clone,
    P6: Parser + Clone,
    P7: Parser + Clone,
{
    type Output = (
        P0::Output,
        P1::Output,
        P2::Output,
        P3::Output,
        P4::Output,
        P5::Output,
        P6::Output,
        P7::Output,
    );

    fn make_seq(self) -> impl Parser<Output = Self::Output> + Clone {
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
