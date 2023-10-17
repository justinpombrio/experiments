from decimal import Decimal

class EDT:
    def __init__(self, logger, decision_exception_table = None):
        """
        logger: log.Logger
        decision_exception_table: { (agent_name, decision_name): action }
        """
        self.logger = logger
        if decision_exception_table is None:
            self.decision_exception_table = {}
        else:
            self.decision_exception_table = decision_exception_table

    def name():
        return "edt"

    def decide(self, scenario, decision_name, sim):
        agent_name, actions = scenario.decision_table[decision_name]

        if self.decision_exception_table:
            with self.logger.group(f"This is EDT with the following decision exception table:"):
                for (agent_name, decision_name), action in self.decision_exception_table.items():
                    self.logger.log(f"{agent_name}, {decision_name} -> {action}")
        else:
            self.logger.log(f"This is EDT with an empty decision exception table (pure EDT).")

        self.logger.log(f"I am making the decision '{decision_name}' for {agent_name}.")
        if (agent_name, decision_name) in self.decision_exception_table:
            action = self.decision_exception_table[agent_name, decision_name]
            self.logger.log(f"It's in my decision exception table, so I will {action}")
            return action

        action_to_utility = {}
        with self.logger.group(f"It's not in my decision exception table, so I will consider each possible action:"):
            for action in actions:
                decision_exception_table = self.decision_exception_table.copy()
                decision_exception_table[agent_name, decision_name] = action
                theory = EDT(self.logger, decision_exception_table)
                decision_proc = theory.decide
                start_event = scenario.events[scenario.start_event]
                stop = lambda event: event.label == "decide" and event.decision_name == decision_name
                with self.logger.group(f"Simulating the effects of having had '{agent_name}, {decision_name} -> {action}' in my decision exception table, up to the current moment:"):
                    distr = sim.simulate(decision_proc, decision_proc, scenario, start_event, stop)
                with self.logger.group(f"Now simulating the expected outcome from this moment on:"):
                    expected_utility = Decimal(0.0)
                    for event, prob in distr.items():
                        with self.logger.group(f"Considering a possibility:"):
                            outcome = sim.simulate(decision_proc, decision_proc, scenario, event)
                            expected_utility += prob * outcome[agent_name]
                    self.logger.log(f"Total expected utility: {expected_utility}")
                    action_to_utility[action] = expected_utility

        with self.logger.group(f"Tabulating the results:"):
            for action, utility in action_to_utility.items():
                self.logger.log(f"{action} -> {utility}")

        best_action = pick_best_action(action_to_utility)
        self.logger.log(f"Thus the best action is to {best_action}.")
        return best_action

def pick_best_action(action_to_utility_map):
    best_action = None
    best_utility = None
    for action, utility in action_to_utility_map.items():
        if best_utility is None or utility > best_utility:
            best_utility = utility
            best_action = action
    return best_action
