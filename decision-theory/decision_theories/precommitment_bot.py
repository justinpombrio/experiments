class PrecommitmentBot:
    def all_possible_bots(logger, decision_table):
        all_precommitments = [{}]
        for decision_name, (agent_name, actions) in decision_table.items():
            new_precommitments = []
            for action in actions:
                for precommitment in all_precommitments:
                    new_precommitment = precommitment.copy()
                    new_precommitment[agent_name, decision_name] = action
                    new_precommitments.append(new_precommitment)
            all_precommitments = new_precommitments
        return [
            PrecommitmentBot(logger, precommitment)
            for precommitment in all_precommitments
        ]

    def __init__(self, logger, precommitments):
        """
        logger: log.Logger
        precommitments: { (agent_name, decision_name): action }
        """
        self.logger = logger
        self.precommitments = precommitments

    def name():
        return "PrecommitmentBot"

    def description(self):
        assignment = ", ".join([
            f"{agent_name}/{decision_name}->{action}"
            for (agent_name, decision_name), action in self.precommitments.items()
        ])
        return f"[{assignment}]"
    
    def decide(self, scenario, decision_name, _sim):
        agent_name, _actions = scenario.decision_table[decision_name]
        action = self.precommitments[agent_name, decision_name]
        self.logger.log(f"I am precommited to {action}.")
        return action
