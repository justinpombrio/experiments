# Cutting Weighted Trees into Equal Pieces

> You're cutting broccoli. You obviously want to cut it into equally sized
> pieces. You get to make `K` cuts to the stem. Go!

**Optimization Problem:** Given a tree with `N` weighted vertices totalling
weight `W`, make `K` cuts to minimize the maximum weight of the resulting
subtrees. (Or: maximize the minimum weight.)


## Reducing to a Decision Problem

There's a related decision problem:

**Decision Problem:** Given a tree with `N` weighted vertices totalling weight
`W`, can you make `K` or fewer cuts such that every resulting subtree has weight
at most `Max`? (Or: make `K` or more cuts so that subtrees have weight at least
`Min`.)

We can use the decision problem to solve the optimization problem with precision
`ε`. Determine the smallest possible value of `Max` such that the decision
problem is true for `Max` but not for `Max + Wε` using binary search. This
requires `log 1/ε` invocations of the decision problem.

Technically the decision problem only says "yes" or "no", but in practice it
will have a witness we can extract when it says "yes", that solves the
optimization problem.


## Solving the Decision Problem

The decision problem can be solved in time `O(NK²)`.

Assume that the tree is a binary tree. Any arbitrary tree can be viewed as a
binary tree, where the two children in the binary tree are (i) the first child
in the arbitary tree, and (ii) the right sibling in the arbitrary tree.

For each vertex, we're going to compute a mapping from _number of cuts_ we're
allowed to make, to the _smallest possible remaining weight_ that can be
achieved by snipping off that many subtrees below the vertex, where each subtree
has weight at most `Max`. (The "remaining weight" is how much weight is left
after the cut-off subtrees have been removed.) For convenience, we'll include
an edge leaving the vertex, so that you can "cut off" the entire vertex.
For example, consider this tree, with `Max=10`:

       |
       3
      / \
     3   4
        / \
       7   2

This cut set isn't valid because the subtree has weight 13:

       |
       3
      / *
     3   4
        / \
       7   2

This cut set with two cuts is valid, and leaves remaining weight 6:

       |
       3
      / *
     3   4
        * \
       7   2

The full cuts/remaining-weight table is:

    cuts | remaining-weight
    -----+-----------------
      0  | impossible
      1  | impossible
      2  | 6
      3  | 0
      4  | 0
      5  | 0
        ...

The witnesses for this table are:

       |
       3
      / *
     3   4
        * \
       7   2

       *
       3
      / *
     3   4
        * \
       7   2

Each table needs one entry for each number of cuts, up to `K`.

These tables can easily be computed, in a single pass recursively from the
bottom up! In the base case, there's a single vertex and we can cut it or not:

    |
    w

giving table:

    cuts | remaining-weight
    -----+-----------------
       0 | w // or 'impossible' if w>Max
       1 | 0

In the case that there are two children with tables T1 and T2:

      |
      w
     / \
    T1  T2

We can compute the table T for this vertex by:

    // Make `i` cuts to `T1` and `j` cuts to `T2`
    for all k <= K:
        for all i + j = k:
            T[k] = min(T[k], T1[i] + T2[j] + w)

    // Make `i` cuts to `T1` and `j` cuts to `T2` and one cut at the top edge
    for all k <= K - 1:
        for all i + j = k:
            T[k] = min(T[k], T1[i] + T2[j])

(The case where there's one child is similiar but simpler.)

Once you've computed these tables for every vertex in the tree, check the table
at the root for an entry `(k, 0)`. If there is such an entry, the problem is
possible. And the least such entry `k` is the number of cuts required.

## Efficiency

**Running time:** Each binary vertex must do `O(K²)` work, as it's looping over
all possible pairs of values from its childrens' tables, which each have size
`O(K)`. There are `O(N)` binary vertices, giving time `O(NK²)` for the decision
problem, and `O(NK²log(1/ε))` time for the optimization problem.

**Memory:** There's `O(NK²)` memory usage when extracting witnesses, as you need
to store the cut-set for every entry in every table.

## An Optimization

These tables are large! But they're likely to be repetitive. For example, this
table:

    cuts | remaining-weight
    -----+-----------------
       0 | infinity
       1 | infinity
       2 | infinity
       3 | 8
       4 | 6
       5 | 6
       6 | 3
       7 | 3
       8 | 0
       9 | 0
      10 | 0

only really needs these entries because they dominate the others:

    cuts | remaining-weight
    -----+-----------------
       0 | infinity
       3 | 8
       4 | 6
       6 | 3
       8 | 0

After computing each table, it can be compressed. This will reduce the amount of
work required to compute the table of its parent.

Roughly speaking, the size of a table should reduce exponentially as you get
further from the root vertex, which reduces the `K²` factor to `Klog(K)`.

## The Reverse Problem

All of this is for minimizing the maximum subtree weight. You can reverse this
algorithm by:

- Initialize "remaining-weight" to 0 instead of infinity.
- When combining "remaining-weights", combine them with max instead of min.
- There's no bound on `k` in the tables; the effective bound comes from not
  being able to make too many cuts without the pieces being below size `Min`.
