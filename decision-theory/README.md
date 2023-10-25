# Decision Theory Simulator

This is an XML language that can express common decision theory dilemmas, and a
Python simulator that can run a decision theory on a dilemma.

- [Brief Introduction to Decision Theory](#brief-introduction-to-decision-theory):
  skip this if you know what decision theory is
- [The Dilemma Language](#the-dilemma-language): an XML language for expressing
  decision theory problems
- [Decision Theories](#decision-theories): the interface for defining a
  decision theory, and the algorithms I used for CDT, EDT, UDT, and one more
- [Logging](#logging) for an example of a trace that's printed, showing the
  reasoning of a decision theory on Newcomb's Problem
- [Dilemma Catalog](#dilemma-catalog): a brief description of the dilemmas I've
  written so far, and how each decision theory behaves on them
- [Usage](#usage): how to install and run DTS.

## Brief Introduction to Decision Theory

Decision theory asks the question:

> Given that you have an accurate (probabilistic) model of the world including
> the consequences of your actions, and given also that you have totally
> ordered preferences over possible outcomes, how do you decide which action to
> take?

The answer, of course, is that you should take the action that yields the best
outcome, according to your preferences. Bam! Decision theory solved.

Ok, well it turns out that even with perfect information, determining the
consequences of your decisions gets difficult, especially as decision theorists
like to come up with tricky dilemmas that break reasonable ways of making
decisions. One such tricky dilemma is Newcomb's Problem. Quoting from Wikipedia:

> There is a reliable predictor, another player, and two boxes designated A and
> B. The player is given a choice between taking only box B or taking both
> boxes A and B. The player knows the following:
>
> - Box A is transparent and always contains a visible $1,000.
> - Box B is opaque, and its content has already been set by the predictor:
>
>     - If the predictor has predicted that the player will take both boxes A
>       and B ("two-box"), then box B contains nothing.
>     - If the predictor has predicted that the player will take only box B
>       ("one-box"), then box B contains $1,000,000.
>
> The player does not know what the predictor predicted or what box B contains
> while making the choice.

The argument in favor of two-boxing is clear: the boxes are _here_, in this
room with you. Your action _now_ cannot possibly effect the contents of Box B
because it's physically here in front of you. Either it has $1000,000 in it or
it doesn't. Either way, you get an additional $1000 by taking both boxes.

The argument in favor of one-boxing is just as clear: supposing the predictor
has administered this game before, everyone that one-boxed walked away with
$1000,000 and everyone that two-boxed walked away with $1000. Which category
would you like to be in?

Hopefully it's becoming clear that it might not be trivial to compute the
"best" action, and that there might even be some disagreement over which action
that is.

As an aside: why should you care about unrealistic situations like this? The
world isn't filled with infallible predictors, and if it was we wouldn't let
them run game shows. (Ok Japan might.) Two reasons you should care, though.
First, there are somewhat more realistic dilemmas involving blackmail. Second,
Newcomb's Problem is a _thought experiment_. The point isn't to be realistic,
the point is to stress test your mechanism for making decisions, and make sure
it doesn't explode in fire when put in an unusual situation.

More reading:

- [The Functional Decision Theory (FDT) paper](https://arxiv.org/pdf/1710.05060.pdf).
  See also its citations, though I suspect this paper will be a more readable
  introduction than most other possible entry points into the literature.
- [Towards a New Decision Theory (UDT)](https://www.lesswrong.com/posts/de3xjFaACCAk6imzv/towards-a-new-decision-theory)
- [Timeless Decision Theory: Problems I Can't Solve](https://www.lesswrong.com/posts/c3wWnvgzdbRhNnNbQ/timeless-decision-theory-problems-i-can-t-solve) for tricky decision theory questions
- [Newcomb's problem](https://en.wikipedia.org/wiki/Newcomb%27s_paradox)
- [Causal decision theory (CDT)](https://en.wikipedia.org/wiki/Causal_decision_theory)
- [Evidential decision theory (EDT)](https://en.wikipedia.org/wiki/Evidential_decision_theory)

## The Dilemma Language

The valuable thing I've done is invented a language capable of expressing
decision theory problems precisely. And then implemented an interpreter for
them in Python, because once you have a precise language you can do that.

Let's walk through the dilemma language using Newcomb's Problem as an example,
since it's about as simple as it can be while using nearly every feature of the
language. Here's the full dilemma. Though note that in this case, the predictor
is fallible, and 1% of the time will predict one-boxing without cause, and 1%
of the time will predict two-boxing without cause.

```xml
<dilemma name="Newcomb's Problem" xml-author="Justin Pombrio">
    <description>...</description>

    <scenario start="yesterday">
        <agent>Alice</agent>
            
        <event> 
            <random id="prediction">
                <case prob="0.98">
                    <predict agent="Alice" decision="one-or-two-box" id="correct-prediction">
                        <scenario start="yesterday">
                            <agent>Alice</agent>
                            <event>yesterday</event>
                            <event>box-B-full</event>
                            <event>box-B-empty</event>         
                        </scenario>
                        <case action="one-box">
                            <goto event="box-B-full" id="correct-prediction-one-box"/>
                        </case>
                        <case action="two-box">
                            <goto event="box-B-empty" id="correct-prediction-two-box"/>
                        </case>                                                     
                    </predict>
                </case>
                <case prob="0.01">
                    <goto event="box-B-full" id="wrong-prediction-one-box"/>
                </case>
                <case prob="0.01">                
                    <goto event="box-B-empty" id="wrong-prediction-two-box"/>
                </case>                                               
            </random>
        </event>
                               
        <event>
            <decide agent="Alice" decision="one-or-two-box" id="box-B-full">
                <case action="one-box">
                    <outcome id="one-box-when-box-B-full">
                        <utility agent="Alice" amount="1000000"/>
                    </outcome>
                </case>
                <case action="two-box">
                    <outcome id="two-box-when-box-B-full">
                        <utility agent="Alice" amount="1001000"/>
                    </outcome>
                </case>
            </decide>
        </event>

        <event>
            <decide agent="Alice" decision="one-or-two-box" id="box-B-empty">
                <case action="one-box">
                    <outcome id="one-box-when-box-B-empty">
                        <utility agent="Alice" amount="0"/>
                    </outcome>
                </case>
                <case action="two-box">
                    <outcome id="two-box-when-box-B-empty">
                        <utility agent="Alice" amount="1000"/>
                    </outcome>
                </case>
            </decide>
        </event>
    </scenario>
</dilemma>
```

Let's walk through this piece by piece.

### Metadata

```xml
<dilemma name="Newcomb's Problem" xml-author="Justin Pombrio">
    <description>...</description>
    ...
</dilemma>
```

`name` is the name of the dilemma, `xml-author` is the person who wrote the XML
(and thus responsible for its accuracy). `description` is an extended
description of the problem, like the Wikipedia description above, which I've
elided here for brevity.

### Scenario

```xml
<scenario start="yesterday">
    <agent>Alice</agent>
        
    <event> ... </event>
    <event> ... </event>
    <event> ... </event>
</scenario>
```

The `scenario` element says what the dilemma actually is. It has a set of
_agents_ (in this case just Alice), and a set of _events_. (The events are the
things _inside_ the `<event>` tags.) All of the agents will use the _same_
decision theory, and they all know it. There's a `start` event (see the
attribute on `scenario`), which is the id of the event that the scenario starts
with.

Each event forms a tree, with children in the tree being possible futures. For
example, a `<random>` coin-flip event would have two child events: one
representing what will happen when the coin lands on heads, and one for tails.
As another example, a `<choice>` event presents some choice to some agent, and
its child events say what the consequences of each choice are. These events
are nested within each other, giving a tree of all possible futures.

_Every_ event has an `id` attribute, which must be unique. These are for
logging, and for `<goto>` events, described next.

### Goto

A `<goto>` event jumps to a named top-level `<event>`. Its sole purpose is to
allow sharing in the event "tree", allowing it to become a DAG.

A `<goto>` is semantically identical to replacing the `<goto>` with the event
it references. Doing so with every `<goto>` expands the DAG into a tree. So you
can always think of the events as forming a tree.

For example, `<goto event="box-B-full" id="prediction-one-box"/>` jumps to the event
`<event name="box-B-full">...</event>`.

### Outcome

An `<outcome>` event represents a final outcome of the dilemma, giving the
utility that each agent has obtained. Outcomes are leaves of the event tree, so
if any utility was gained along the way it must be accounted for in the
`outcome`s beneath it. Each agent's utility is written with a child element
`<utility agent=AGENT_NAME amount=UTILITY/>`

For example, here's the outcome if Alice two-boxes when box B is full:
```xml
<outcome id="two-box-when-box-B-full">
    <utility agent="Alice" amount="1001000"/>
</outcome>
```

For an example with multiple agents, here's an `outcome` event representing
that "Alice wins $100 and Bob wins $0":

```xml
<outcome id="alice-wins">
    <utility agent="Alice" amount="100"/>
    <utility agent="Bob" amount="0"/>
</outcome>
```

### Random

A `<random>` event represents randomness with a finite set of possible
outcomes. Each outcome is given by a child `<case prob=P>EVENT</case>`, meaning
that `EVENT` happens with probability `P`. The probabilities of the `case`s
must sum to 1.

For example, here's a `random` event representing that "Alice wins $100 if a
coin lands on heads":

```xml
<random id="coin-flip">
    <case prob="0.5">
        <outcome id="heads">
            <utility agent="Alice" amount="100"/>
        </outcome>
    </case>
    <case prob="0.5">
        <outcome id="tails">
            <utility agent="Alice" amount="0"/>
        </outcome>
    </case>
</random>
```

And here's the randomness in the Newcomb problem. With 98% probability, the
predictor will accurately predict Alice's decision, with 1% probability She
will conclude without evidence that Alice will one-box, and with 1% probability
She will conclude without evidence that Alice will two-box:

```xml
<random id="predictor-is-predicting">
    <case prob="0.98">
        <predict agent="Alice" decision="one-or-two-box" id="prediction">
            ...
        </predict>
    </case>
    <case prob="0.01">
        <goto event="box-B-full" id="wrong-prediction-one-box"/>
    </case>
    <case prob="0.01">                
        <goto event="box-B-empty" id="wrong-prediction-two-box"/>
    </case>                                               
</random>
```

### Decide

```xml
<decide agent="Alice" decision="one-or-two-box" id="box-B-full">
    <case action="one-box">
        <outcome id="one-box-when-box-B-full">
            <utility agent="Alice" amount="1000000"/>
        </outcome>
    </case>
    <case action="two-box">
        <outcome id="two-box-when-box-B-full">
            <utility agent="Alice" amount="1001000"/>
        </outcome>
    </case>
</decide>
```

A `<decide>` event represents some agent making some decision.  The `agent`
attribute says which agent is making the decision, and the `decision` attribute
says which decision it is. **Multiple `<decide>` events may have the same
`decision` attribute, meaning that the agent does not know which event she is
on when making the decision.** For example, in the Newcomb problem the agent
doesn't know whether box B is full or not when making the decision
`one-box-or-two-box`. Thus there are two `<decide>` events with
`decision="one-or-two-box"`; one where box B is empty and one where it's full.

### Predict

```xml
<predict agent="Alice" decision="one-or-two-box" id="prediction">
    <scenario start="yesterday">
        <agent>Alice</agent>
        <event>yesterday</event>
        <event>box-B-full</event>
        <event>box-B-empty</event>         
    </scenario>
    <case action="one-box">
        <goto event="box-B-full" id="correct-prediction-one-box"/>
    </case>
    <case action="two-box">
        <goto event="box-B-empty" id="correct-prediction-two-box"/>
    </case>
</predict>
```

`<predict>` is the most complicated event, because the prediction could be
for any scenario at all. It has the following parts:

- A `<scenario>` child gives the scenario being predicted. It has agents and
  events, just like the top-level scenario. The only difference is that its
  events are just names of events from the existing scenario.
- The `agent` and `decision` attributes say which agent and decision is being
  predicted, within the given `scenario`.
- The `case` children give the consequences of each predicted action. In this
  example, the predictor makes box B full iff Alice is predicted to one-box.

### Forced Decide

A `<forced\_decide>` event is like a `<decide>` event, but the agent doesn't
actually get a choice. This is how dilemmas like "smoking lesion" or "cosmic
ray", where the agent makes some decision in a way other than her preference,
are represented. For example, here's a "lesion" forcing Alice to choose
`yes-smoke` in the decision `smoke-or-not`:

```xml
<forced_decide agent="Alice" decision="smoke-or-not" id="affected-by-lesion">
    <case action="yes-smoke">
        <outcome id="forced-to-smoke">
            <utility agent="Alice" amount="-999000"/>
        </outcome>
    </case>
</forced_decide>
```

(I'm not very happy with this representation of forced choices. I'm not sure
it's a faithful way of encoding the smoking lesion problem, and it makes EDT
quite unnatural to write down: it needs to go out of its way to account for
every forced choice and then do the wrong thing for them.)

## Decision Theories

A decision theory has a small interface:

```python
class CDT:
    def __init__(self, logger):
        self.logger = logger

    def name():
        return "cdt"

    def decide(self, scenario, decision_name, sim):
        ...
```

`logger` should be used to provide a trace of the decision theory's reasoning
each time `decide()` is called. `logger` has two methods:

- `logger.log(message)` prints string `message`
- `with logger.group(message):` prints string `message`, and _indents_ all
  logging inside thw `with` block.

`name()` is a unique name for the decision theory. It's what will be used on
the command line to specify this decision theory.

`decide(self, scenario, decision_name, simulator)` will be invoked each time a
decision needs to be made. **It must be stateless and deterministic.** Thus it
cannot call an RNG, and it must not access `self` except for logging and
possibly accessing static data. The three arguments to `decide` are:

- `scenario` gives the full scenario being run. It's given as a `Scenario`
  object, which you can see in `dilemma.py`. It's a representation of the
  `<scenario>` element in the dilemma XML.
- `decision_name` is a string giving the decision name being made. Remember
  that one decision name could refer to multiple `<decide>` events, because
  the agent might not have complete knowledge. You can use the decision name
  to look up the agent making the decision and the allowed actions using
  `agent_name, actions = scenario.decision_table[decision_name]`.
- `simulator` gives access to the dilemma simulator. See `dilemma.py` for docs on
  how to call it. This argument is _technically_ redundant; `scenario` gives
  you all the information you need to run your own simulation, though it
  would be a lot of repeated code. Be careful not to produce infinite loops
  when calling the simulator.

There are four built-in decision theories. I will describe how I implemented
them. I am not certain this is faithful to how they are _supposed_ to behave!
There is also legitimate ambiguity in that CDT and EDT depend on a prior; I've
chosen a uniform prior over all possible dispositions for both of them.

- _CDT, causal decision theory._ First, determine a probability distribution
  over the current event. Do so by starting with a uniform prior over all of
  the possible behaviors for all agents, and simulating forward from there
  until reaching the current decision. Then normalize this probability
  distribution so the probabilities to add to 1 (they could have been less than
  or more than 1). Then, consider each possible action in turn. For each,
  compute the expected utility starting from the probability distribution
  above, given that the agent takes said action and uses CDT from this point
  forward. (Notably, if they encounter the _same_ decision again in the future,
  they do not assume they will make it in the same way. This leads to an
  infinite loop in the Sleeping Beauty problem!) Finally, pick the action with
  the highest expected utility.
- _EPDT, evidential precommitment decision theory._ This is a variant of EDT
  that was more natural to implement in this framework. To make a decision, the
  agent first considers each possible action in turn. For that action, she:
      1. Computes the probability distribution over the current event, under
         the assumption that she precommited to picking this action from the
         start of the scenario.
      2. Normalize this probability distribution so that the probabilities sum
         to 1 (again, they could have summed to less than or more than 1).
      3. Compute the expected utility from this point forward, assuming that
         the agent continues to commit to making this decision the same way.
  Then just pick the action with the highest expected utility.
- _EDT, evidential decision theory._ 
      1. Determine a probability distribution over the current event. Do so by
         starting with a uniform prior over all possible behaviors for all
         agents, and simulating forward from there until reaching the current
         decision.
      2. Re-group the probability distribution based on action.
      3. Normalize this probability distribution so that the probabilities sum
         to 1.
      4. For each action, compute the expected utility from this point forward,
         assuming that all agents continue to behave the same way.
  Then pick the action with the highest utility.
- _UDT, updateless decision theory._ Make each decision the way you would have
  precommited to make it at the start of the scenario. To be more precise, for
  each action, calculate the expected utility if, at the start of the scenario,
  you had precommited to pick that action for the given decision, and continued
  following UDT for other decisions. Then pick the action with the highest
  utility. This is not precisely UDT, but I believe it's equivalent to UDT, and
  to FDT, within the confines of the dilemmas expressible in the XML language.
  At least for single-agent dilemmas. If you know otherwise, or think it should
  be named differently, let me know. (If I were to name it from scratch, I
  would call it "precommitment decision theory".)

(A few of these descriptions used the word "behavior". I used it to mean a
_complete_ mapping from decision to action, which fixes all decisions made by
an agent.)

## Logging

When you run a dilemma using a decision theory, you get a very detailed log of
what's happening, and what the agent's reasoning process is. For example,
here's "UDT" on a version of the Newcomb problem where the predictor is
infallible (it's the same as the one we've walked through, except for missing
the two 1% random cases where the predictor makes a mistake):

```
SIMULATE prediction
|   PREDICT one-or-two-box by Alice from prediction:
|   ?   I, Alice, am deciding one-or-two-box using UDT.
|   ?   Considering possible precommitments:
|   ?       with precommitment 'Alice, one-or-two-box -> one-box':
|   ?           SIMULATE prediction
|   ?           |   PREDICT one-or-two-box by Alice from prediction:
|   ?           |   ?   I, Alice, am deciding one-or-two-box using UDT.
|   ?           |   ?   Precommitments:
|   ?           |   ?       Alice, one-or-two-box -> one-box
|   ?           |   ?   I am precommited to one-box.
|   ?           |   Alice is predicted to decide to one-box
|   ?           |   DO box-B-full:
|   ?           |       DECIDE one-or-two-box by Alice:
|   ?           |       !   I, Alice, am deciding one-or-two-box using UDT.
|   ?           |       !   Precommitments:
|   ?           |       !       Alice, one-or-two-box -> one-box
|   ?           |       !   I am precommited to one-box.
|   ?           |       Alice decides to one-box
|   ?           |       OUTCOME:
|   ?           |           Alice -> 1,000,000
|   ?       with precommitment 'Alice, one-or-two-box -> two-box':
|   ?           SIMULATE prediction
|   ?           |   PREDICT one-or-two-box by Alice from prediction:
|   ?           |   ?   I, Alice, am deciding one-or-two-box using UDT.
|   ?           |   ?   Precommitments:
|   ?           |   ?       Alice, one-or-two-box -> two-box
|   ?           |   ?   I am precommited to two-box.
|   ?           |   Alice is predicted to decide to two-box
|   ?           |   DO box-B-empty:
|   ?           |       DECIDE one-or-two-box by Alice:
|   ?           |       !   I, Alice, am deciding one-or-two-box using UDT.
|   ?           |       !   Precommitments:
|   ?           |       !       Alice, one-or-two-box -> two-box
|   ?           |       !   I am precommited to two-box.
|   ?           |       Alice decides to two-box
|   ?           |       OUTCOME:
|   ?           |           Alice -> 1,000
|   ?   Expected utility for each precommitment:
|   ?       one-box -> 1,000,000
|   ?       two-box -> 1,000
|   ?   Thus my best action is to one-box.
|   Alice is predicted to decide to one-box
|   DO box-B-full:
|       DECIDE one-or-two-box by Alice:
|       !   I, Alice, am deciding one-or-two-box using UDT.
|       !   Considering possible precommitments:
|       !       with precommitment 'Alice, one-or-two-box -> one-box':
|       !           SIMULATE prediction
|       !           |   PREDICT one-or-two-box by Alice from prediction:
|       !           |   ?   I, Alice, am deciding one-or-two-box using UDT.
|       !           |   ?   Precommitments:
|       !           |   ?       Alice, one-or-two-box -> one-box
|       !           |   ?   I am precommited to one-box.
|       !           |   Alice is predicted to decide to one-box
|       !           |   DO box-B-full:
|       !           |       DECIDE one-or-two-box by Alice:
|       !           |       !   I, Alice, am deciding one-or-two-box using UDT.
|       !           |       !   Precommitments:
|       !           |       !       Alice, one-or-two-box -> one-box
|       !           |       !   I am precommited to one-box.
|       !           |       Alice decides to one-box
|       !           |       OUTCOME:
|       !           |           Alice -> 1,000,000
|       !       with precommitment 'Alice, one-or-two-box -> two-box':
|       !           SIMULATE prediction
|       !           |   PREDICT one-or-two-box by Alice from prediction:
|       !           |   ?   I, Alice, am deciding one-or-two-box using UDT.
|       !           |   ?   Precommitments:
|       !           |   ?       Alice, one-or-two-box -> two-box
|       !           |   ?   I am precommited to two-box.
|       !           |   Alice is predicted to decide to two-box
|       !           |   DO box-B-empty:
|       !           |       DECIDE one-or-two-box by Alice:
|       !           |       !   I, Alice, am deciding one-or-two-box using UDT.
|       !           |       !   Precommitments:
|       !           |       !       Alice, one-or-two-box -> two-box
|       !           |       !   I am precommited to two-box.
|       !           |       Alice decides to two-box
|       !           |       OUTCOME:
|       !           |           Alice -> 1,000
|       !   Expected utility for each precommitment:
|       !       one-box -> 1,000,000
|       !       two-box -> 1,000
|       !   Thus my best action is to one-box.
|       Alice decides to one-box
|       OUTCOME:
|           Alice -> 1,000,000
Final outcome:
    Alice -> 1,000,000
```

The vertical lines give context:

- `|` marks simulations. The outer `|` is what's actually happening; inner ones
  are from agents simulating hypotheticals in their minds (or, perhaps, on
  paper).
- `?` marks the predictor predicting what an agent _would_ do.
- `!` marks agents actually reasoning to make a decision.

## Dilemma Catalog

TODO

Current issues:

- CDT infinite loops on prisoner's dilemma

## Usage

### Installation

You'll need Python 3, and two libraries:

    pip install argparse
    pip install xmlschema

### Running

To validate a dilemma:

    python dts.py validate dilemmas/newcomb.xml

To run a dilemma using a decision theory:

    python dts.py run dilemmas/newcomb.xml cdt

The decision theories available can be seen in `dts.py`:

    DECISION_THEORY_LIST = [CDT, EDT, EPDT, UDT]

(See disclaimers about these decision theories in
[decision-theories](#decision-theories). Tldr; these were my best first
attempts, but may not be faithful to how these decision theories are "supposed"
to operate.)
