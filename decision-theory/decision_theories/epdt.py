from decimal import Decimal

# Evidential Precommitment Decision Theory

class EPDT:
    def __init__(self, logger, precommitments = None):
        """
        logger: log.Logger
        precommitments: { (agent_name, decision_name): action }
        """
        self.logger = logger
        if precommitments is None:
            self.precommitments = {}
        else:
            self.precommitments = precommitments

    def name():
        return "epdt"

    def decide(self, scenario, decision_name, sim):
        agent_name, actions = scenario.decision_table[decision_name]

        self.logger.log(f"I, {agent_name}, am deciding {decision_name} using EPDT.")
        if self.precommitments:
            with self.logger.group("Precommitments:"):
                for (agent, decision), action in self.precommitments.items():
                    self.logger.log(f"{agent}, {decision} -> {action}")

        if (agent_name, decision_name) in self.precommitments:
            action = self.precommitments[agent_name, decision_name]
            self.logger.log(f"I am precommited to {action}.")
            return action

        action_to_utility = {}
        with self.logger.group(f"Considering possible precommitments:"):
            for action in actions:
                with self.logger.group(f"with precommitment '{agent_name}, {decision_name} -> {action}':"):
                    precommitments = self.precommitments.copy()
                    precommitments[agent_name, decision_name] = action
                    theory = EPDT(self.logger, precommitments)
                    decision_proc = theory.decide
                    start_event = scenario.events[scenario.start_event]
                    def stop(event):
                        return ((event.label == "decide" or event.label == "forced_decide")
                            and event.decision_name == decision_name)
                    distr = sim.simulate(decision_proc, decision_proc, scenario, start_event, stop)

                    with self.logger.group(f"Probability distribution of my current event:"):
                        for event, prob in distr.items():
                            self.logger.log(f"  {event.id} -> {prob}")

                    normal_distr = normalize_distribution(distr)
                    with self.logger.group(f"Normalized probability distribution of my current event:"):
                        for event, prob in normal_distr.items():
                            self.logger.log(f"  {event.id} -> {prob}")

                    with self.logger.group(f"Computing utility starting from my current event:"):
                        expected_utility = Decimal(0.0)
                        for event, prob in normal_distr.items():
                            with self.logger.group(f"Considering possibility {event.id}:"):
                                outcome = sim.simulate(decision_proc, decision_proc, scenario, event)
                                expected_utility += prob * outcome[agent_name]
                        self.logger.log(f"Total expected utility: {expected_utility}")
                        action_to_utility[action] = expected_utility

        with self.logger.group(f"Expected utility for each precommitment:"):
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
