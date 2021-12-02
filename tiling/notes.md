# Notes

### Normal Maps

Conversion from point on sphere to normal map color:

    def c(x): return 127. * x + 128.
    def h(x): return hex(int(c(x)))
    def f(x, y, z): return [h(x), h(y), h(z)]

Normal map colors:

                *
        *               E
                *
            *       D

    *     *     A     B     C

           *        *
                *
        *               *
                *

    A:  0   0   1    -> #8080ff
    B:  0.7 0   0.7  -> #d980d9
    C:  1   0   0    -> #ff8080
    D:  0.5 0.5 0.7  -> #bfbfd9
    E:  0.7 0.7 0    -> #d9d980
