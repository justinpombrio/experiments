A common pattern is to use a logger _all over_ your codebase. This requires
either a global mutable variable (which is magical and a little hard to test),
or passing the logger to nearly every function in the codebase (which is a lot
of boilerplate).

The idea behind `provide/require` (a.k.a. implicit) is to make a type checked
version of "dynamic extent". One part of the code provides a value:

    fn main() {
        provide logger = Logger::new_stdout();
        start();
    }

Another part of the code says what it `requires` and can use it for free:

    fn start() requires logger: Logger {
        step_1();
        step_2();
        step_3();
    }

    fn step_2() requires logger: Logger {
        ...
        logger.log("All good!");
        ...
    }

Importantly, you don't have to explicitly pass `requires` from one function to
another. Going even further, you can add `requires` to the entire module:

    module steps requires logger: Logger;

    fn step_2() {
        ...
        logger.log("All good!");
        ...
    }

Related: implicits https://www.microsoft.com/en-us/research/wp-content/uploads/2019/03/implicits-tr-v1.pdf
