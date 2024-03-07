# Cutting Weighted Trees into Equal Pieces

> You're cutting broccoli. You obviously want to cut it into equally sized
> pieces. You get to make `K` cuts to the stem. Go!

**Optimization Problem:** Given a tree with `N` weighted vertices totalling
weight `W`, make `K` cuts to minimize the maximum weight of the resulting
subtrees. (Or: maximize the minimum weight.)

Tldr: this can be solved with a simple algorithm (55 lines of code) in
essentially linear time (finding the optimal cuts is faster than printing the
tree).

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

Technically the decision problem only says "yes" or "no", but in practice, when
it says "yes" it will be able to produce a witness partition that solves the
optimization problem.


## Solving the Decision Problem

The decision problem can be solved in time `O(N)`.

For each vertex, compute the minimal set of cuts required such that the
resulting subtrees have weight at most `Max`, while minimizing the weight of the
subtree at the root.

For example, consider this tree, with `Max=10`:

       3
      / \
     3   4
        / \
       7   2

This cut set isn't valid because the subtree has weight 13:

       3
      / *
     3   4
        / \
       7   2

    invalid

This cut set with two cuts is valid, and leaves remaining weight 9:

       3
      * \
     3   4
        * \
       7   2

    num-cuts: 2
    remaining-weight: 6

This cut set _also_ has two cuts, but it leaves a smaller amount of remaining
weight (6), so it is preferable:

       3
      / *
     3   4
        * \
       7   2
    
    num-cuts: 2
    remaining-weight: 6

This can easily be computed in a single pass recursively from the bottom up! In
the base case, there are no cuts and the remaining weight is the vertex's
weight. Unless the vertex weights too much!

    w

    if w > Max:
        return 'impossible'
    else:
        return:
            num-cuts = 0
            remaining-weight = w

In the case that there are two children A and B:

       w
      / \
     A   B

    if w > Max:
        return 'impossible'
    let num-cuts = A.num-cuts + B.num-cuts
    let remaining-weight = w

    WLOG assume A.remaining-weight <= B.remaining-weight
      Otherwise swap A and B.

    if remaining-weight + A.remaining-weight <= Max:
        remaining-weight += A.remaining-weight
    else:
        cut the edge between w and A
        num-cuts += 1

    if remaining-weight + B.remaining-weight <= Max:
        remaining-weight += B.remaining-weight
    else:
        cut the edge between w and B
        num-cuts += 1

    return
        num-cuts
        remaining-weight

For more children, do the same thing after sorting the children by their
remaining weights.

Once you've computed this, `num-cuts` at the root node gives the minimum number
of cuts required for the entire tree.

## Efficiency

**Running time:** The decision problem does `O(D log(D))` work for each vertex,
where `D` is the degree of that vertex (just because it has to sort a small list
of integers). Amortized, that's `O(log(D))` per vertex where `D` is the maximum
degree of any vertex. There are `O(N)` vertices, giving total time `O(N log D)`
for the decision problem. The optimization problem invokes the decision problem
`log(1/ε)` times, giving a final time complexity of `O(N log(D) log(1/ε))`
where `N` is the tree size, `D` is the maximum vertex degree, and `ε` is the
(unitless) desired precision.

**Memory:** Assuming you sort in place and mark the tree cuts in place, the
memory usage is `O(1)`.

**In practice:** The implementation can find the optimal partition of a tree
with 1,000,000 vertices using 10,000 cuts in a few seconds. Most of the running
time is spent printing the tree.

## The Reverse Problem

All of this is for minimizing the maximum subtree weight. You might also want to
maximize the minimum subtree weight. I suspect this can also be done in ~linear
time but that it's a bit trickier.

## Implementation

The implementation is very fast and extensively tested.

- `src/tree.rs` is a straightforward tree implementation. For convenience, the
  tree itself stores a set of cuts.
- `src/orcacle.rs` is an "oracle implementation". It simply tries _every_ set of
  cuts and picks the best one: thus it is obviously correct but has exponential
  running time.
- `src/generator.rs` is an implementation of generators for testing purposes.
  It's used to e.g. generate every tree up to a given size without repetition in
  linear time.
- `src/min_of_max.rs` is the implementation of this algorithm (only 55 LOC)
  and a test case. The test case has verified that the implementation agrees
  with the oracle for every tree with positive-integer-weighted vertices of
  total weight up to 12 and every number of cuts up to 6. (The test case as
  committed does a smaller check so that it runs more quickly.)
- `src/max_of_min.rs` is the implementation of the reverse algorithm. It's more
  complicated, at 85 LOC. It has also been tested for trees up to weight 12 with
  up to 6 cuts.
- `src/main.rs` will find the optimal set of 10,000 cuts for a tree with 1,000,000
  nodes, to ensure that the implementation runs quickly.

## Usage

Install Rust with:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Follow the instructions at the end for how to add `cargo` to the current
terminal path.

Build with:

    bash build.sh

(It's just shorthand for the following):

    cargo build --release
    cp target/release/tree-cut tree-cut

Run with:

    ./tree-cut

To see options:

    ./tree-cut --help
