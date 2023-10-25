class UDT:
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
        return "udt"

    def decide(self, scenario, decision_name, sim):
        agent_name, actions = scenario.decision_table[decision_name]

        self.logger.log(f"I, {agent_name}, am deciding {decision_name} using UDT.")
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
                precommitments = self.precommitments.copy()
                precommitments[agent_name, decision_name] = action
                theory = UDT(self.logger, precommitments)
                decision_proc = theory.decide
                start_event = scenario.events[scenario.start_event]
                with self.logger.group(f"with precommitment '{agent_name}, {decision_name} -> {action}':"):
                    outcome = sim.simulate(decision_proc, decision_proc, scenario, start_event)
                action_to_utility[action] = outcome[agent_name]

        with self.logger.group(f"Expected utility for each precommitment:"):
            for action, utility in action_to_utility.items():
                self.logger.log(f"{action} -> {utility:,}")

        best_action = pick_best_action(action_to_utility)
        self.logger.log(f"Thus my best action is to {best_action}.")
        return best_action

def pick_best_action(action_to_utility_map):
    best_action = None
    best_utility = None
    for action, utility in action_to_utility_map.items():
        if best_utility is None or utility > best_utility:
            best_utility = utility
            best_action = action
    return best_action
