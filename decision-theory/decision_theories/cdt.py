from decimal import Decimal

class CDT:
    def __init__(self, logger):
        self.logger = logger

    def name():
        return "cdt"

    def decide(self, scenario, decision_name, sim):
        agent_name, actions = scenario.decision_table[decision_name]

        self.logger.log(f"I, {agent_name}, am deciding {decision_name} using CDT.")

        # How did I get here, to this moment in my life?
        # Who knows! Let's just assume everyone acted and predicted randomly.
        def decide_randomly(scenario, decision_name, sim):
            _, actions = scenario.decision_table[decision_name]
            prob = Decimal(1.0) / Decimal(len(actions))
            return {
                action: prob
                for action in actions
            }
        start_event = scenario.events[scenario.start_event]
        stop = lambda event: event.label == "decide" and event.decision_name == decision_name
        distr = sim.simulate(decide_randomly, decide_randomly, scenario, start_event, stop)

        with self.logger.group(f"Probability distribution of my current situation:"):
            for event, prob in distr.items():
                self.logger.log(f"  {event.id} -> {prob}")

        normal_distr = normalize_distribution(distr)
        with self.logger.group(f"Normalized probability distribution of my current situation:"):
            for event, prob in normal_distr.items():
                self.logger.log(f"  {event.id} -> {prob}")

        action_to_utility = {}
        with self.logger.group(f"Considering consequences of possible actions:"):
            for action in actions:
                with self.logger.group(f"with action '{action}':"):
                    with self.logger.group(f"Computing utility starting from my current situation:"):
                        expected_utility = Decimal(0.0)
                        for event, prob in normal_distr.items():
                            with self.logger.group(f"for possibility {event.id}:"):
                                event_from_action = event.cases[action]
                                outcome = sim.simulate(self.decide, self.decide, scenario, event_from_action)
                                expected_utility += prob * outcome[agent_name]
                        self.logger.log(f"Total expected utility: {expected_utility}")
                        action_to_utility[action] = expected_utility

        with self.logger.group(f"Expected utility for each action:"):
            for action, utility in action_to_utility.items():
                self.logger.log(f"{action} -> {utility:,}")

        best_action = pick_best_action(action_to_utility)
        self.logger.log(f"Thus my best action is to {best_action}.")
        return best_action

def normalize_distribution(distr):
    total_prob = Decimal(0.0)
    for prob in distr.values():
        total_prob += prob
    return {
        key : prob / total_prob
        for key, prob in distr.items()
    }

def pick_best_action(action_to_utility_map):
    best_action = None
    best_utility = None
    for action, utility in action_to_utility_map.items():
        if best_utility is None or utility > best_utility:
            best_utility = utility
            best_action = action
    return best_action
