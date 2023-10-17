# TODO: decimal.Decimal?

# Local modules
from parse import Scenario

# Standard library
from decimal import Decimal

class Simulator:
    def __init__(self, logger):
        self.logger = logger

    def simulate(self, decide, predict, scenario, event, stop=None):
        """
        Simulate `event` within `scenario`, in which decisions are made via the
        `decide` procedure and predictions are made via the `predict`
        procedure.

        - `scenario`: a `Scenario` describing the context in which `event`
          happens
        - `event`: an event object (Do, Random, Predict, Decide, or Outcome)
          within the scenario
        - `decide`: a function that determines how agents make decisions. It
          takes as arguments:
              Scenario, agent_name, decision_name, Simulator
          and returns either the action to take (`str`), or a probability
          distribution over actions (`{str: float}`). The `Simulator` argument
          allows the decision theory to recursively call this simulation method.
          However, watch out for infinite loops!
        - `predict`: a function that determines what agents are predicted to
          do. Accepts the same type signatures as `decide`.
        - If `stop` is None, returns `{ agent_name : expected_utility }`
          (a map from agent name to expected utility for that agent).
        - If `stop` is a `event -> bool`, returns `{ event : probability }`
          (a map from event object to probability that that event occurs).
          Note that the probabilities for events might add up to less than or
          more than 1.
        """

        if event.label == "do":
            # TODO: cycle detection
            with self.logger.group(f"Event '{event.event_name}':"):
                event = scenario.events[event.event_name]
                outcome = self.simulate(decide, predict, scenario, event, stop)

        elif event.label == "random":
            outcome = {} # map from agent name to expected utility
            with self.logger.group("Random event:"):
                for prob, case in event.cases:
                    with self.logger.group(f"With probability {prob}:"):
                        conditional_outcome = self.simulate(decide, predict, scenario, case, stop)
                        for key, val in conditional_outcome.items():
                            outcome.setdefault(key, Decimal(0.0))
                            outcome[key] += prob * val
                if stop is None:
                    with self.logger.group("Average outcome:"):
                        for agent, utility in outcome.items():
                            self.logger.log(f"{agent} -> {utility}")

        elif event.label == "predict":
            new_scenario = Scenario(
                event.agent_names,
                {
                    event_name: case
                    for event_name, case in scenario.events.items()
                    if event_name in event.event_names
                },
                event.start_event
            )
            with self.logger.group(f"Predicting {event.agent_name}'s making decision '{event.decision_name}' in scenario '{event.start_event}':"):
                action = predict(new_scenario, event.decision_name, self)
            outcome = self.do_action(decide, predict, scenario, event, stop, action)

        elif event.label == "decide":
            with self.logger.group(f"{event.agent_name} is making decision '{event.decision_name}':"):
                action = decide(scenario, event.decision_name, self)
            outcome = self.do_action(decide, predict, scenario, event, stop, action)

        elif event.label == "outcome":
            if stop is not None:
                self.logger.log("Ignoring outcome")
                outcome = {}
            else:
                with self.logger.group("Outcome"):
                    outcome = event.utilities
                    for agent, utility in outcome.items():
                        self.logger.log(f"{agent} -> {utility}")

        else:
            raise Exception(f"Bug! Invalid label '{label}'")

        if stop is not None and stop(event):
            outcome.setdefault(event, Decimal(0.0))
            outcome[event] += Decimal(1.0)

        return outcome

    def do_action(self, decide, predict, scenario, event, stop, action):
        verb = "chooses to" if event.label == "prediction" else "is predicted to"
        adj = "predicted " if event.label == "prediction" else ""
        agent = scenario.decision_table[event.decision_name][0]

        if type(action) is str:
            # 1. Log action
            self.logger.log(f"{agent} {verb} {action}")
            # 2. Validate action
            if action not in event.cases:
                self.logger.log(f"{agent} {adj}decision is invalid!")
                return None
            # 3. Compute outcome from action
            return self.simulate(decide, predict, scenario, event.cases[action], stop)

        else:
            # 1. Log action probability distribution
            action_distr = action
            with self.logger.group(f"{agent} {verb}:"):
                for act, prob in action_distr.items():
                    self.logger.log(f"{act}, with probability {prob}")
            # 2. Validate possible actions
            for act, _ in action_distr.items():
                if act not in event.cases:
                    self.logger.log(f"{agent} {adj}decision (\"{act}\") is invalid!")
                    return None
            # 3. Compute expected outcome from action probability distribution
            outcome = {}
            for act, prob in action_distr.items():
                verb = "is predicted to" if event.label == "prediction" else "will"
                with self.logger.group(f"{agent} {verb} {act} with probability {prob}:"):
                    conditional_outcome = self.simulate(decide, predict, scenario, event.cases[act], stop)
                    for key, val in conditional_outcome:
                        outcome.setdefault(key, Decimal(0.0))
                        outcome[key] += prob * val
            return outcome
