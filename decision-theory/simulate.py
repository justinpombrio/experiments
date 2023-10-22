# TODO: decimal.Decimal?

# Local modules
from dilemma import Scenario

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
        - `event`: an event object (Goto, Random, Predict, Decide, or Outcome)
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
        with self.logger.group(f"SIMULATE {event.id}", "|"):
            result = self.__sim(decide, predict, scenario, event, stop)
        return result

    def simulate_with_distribution(self, distribution, scenario, event, stop=None):
        """
        distribution: [(prob, decide, predict, name)]

        Run `simulate()` with a probability distribution over `(decide, predict)`
        pairs, each with a `name` for logging. Return the _expected_ result.
        """

        expected_outcome = {}
        for prob, decide, predict, name in distribution:
            with self.logger.group(f"SIMULATE {event.id} using {name}", "|"):
                outcome = self.__sim(decide, predict, scenario, event, stop)
                for key, val in outcome.items():
                    expected_outcome.setdefault(key, Decimal(0.0))
                    expected_outcome[key] += prob * val
        return expected_outcome

    def __sim(self, decide, predict, scenario, event, stop=None):

        if event.label == "goto":
            # TODO: cycle detection
            with self.logger.group(f"DO {event.event_name}:"):
                inner_event = scenario.events[event.event_name]
                outcome = self.__sim(decide, predict, scenario, inner_event, stop)

        elif event.label == "random":
            outcome = {} # map from agent name to expected utility
            with self.logger.group("RANDOM:"):
                for prob, case in event.cases:
                    with self.logger.group(f"with probability {prob}:"):
                        conditional_outcome = self.__sim(decide, predict, scenario, case, stop)
                        for key, val in conditional_outcome.items():
                            outcome.setdefault(key, Decimal(0.0))
                            outcome[key] += prob * val
                if stop is None:
                    with self.logger.group("Average outcome:"):
                        for agent, utility in outcome.items():
                            self.logger.log(f"{agent} -> {utility:,}")

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
            with self.logger.group(f"PREDICT {event.decision_name} by {event.agent_name} from {event.start_event}:", "?"):
                action = predict(new_scenario, event.decision_name, self)
            agent = scenario.decision_table[event.decision_name][0]
            self.logger.log(f"{agent} is predicted to decide to {action}")
            if action not in event.cases:
                raise Exception(f"{agent}'s prediction to {action} is invalid!")
            outcome = self.__sim(decide, predict, scenario, event.cases[action], stop)

        elif event.label == "decide":
            with self.logger.group(f"DECIDE {event.decision_name} by {event.agent_name}:", "!"):
                action = decide(scenario, event.decision_name, self)
            agent = scenario.decision_table[event.decision_name][0]
            self.logger.log(f"{agent} decides to {action}")
            if action not in event.cases:
                raise Exception(f"{agent}'s decision to {action} is invalid!")
            outcome = self.__sim(decide, predict, scenario, event.cases[action], stop)

        elif event.label == "forced_decide":
            agent = scenario.decision_table[event.decision_name][0]
            self.logger.log(f"FORCED_DECIDE. {agent} forced to decide to {event.action}.")
            outcome = self.__sim(decide, predict, scenario, event.case, stop)

        elif event.label == "outcome":
            with self.logger.group("OUTCOME:"):
                for agent, utility in event.utilities.items():
                    self.logger.log(f"{agent} -> {utility:,}")
                if stop is None:
                    outcome = event.utilities
                else:
                    outcome = {}

        else:
            raise Exception(f"Bug! Invalid label '{label}'")

        if stop is not None and stop(event):
            outcome.setdefault(event, Decimal(0.0))
            outcome[event] += Decimal(1.0)
            self.logger.log(f"Yielding event {event.id}.")

        return outcome

        # Code for agents behaving randomly. Unclear how prediction should work
        # in this case, though.
        #
        #else:
        #    # 1. Log action probability distribution
        #    action_distr = action
        #    with self.logger.group(f"{agent} {verb}:"):
        #        for act, prob in action_distr.items():
        #            self.logger.log(f"{act}, with probability {prob}")
        #    # 2. Validate possible actions
        #    for act, _ in action_distr.items():
        #        if act not in event.cases:
        #            raise Exception(f"{agent} {adj}decision to {act} is invalid!")
        #    # 3. Compute expected outcome from action probability distribution
        #    outcome = {}
        #    for act, prob in action_distr.items():
        #        verb = "is predicted to" if event.label == "prediction" else "will"
        #        with self.logger.group(f"{agent} {verb} {act} with probability {prob}:"):
        #            conditional_outcome = self.__sim(decide, predict, scenario, event.cases[act], stop)
        #            for key, val in conditional_outcome.items():
        #                outcome.setdefault(key, Decimal(0.0))
        #                outcome[key] += prob * val
        #    return outcome
