from decimal import Decimal
from decision_theories.precommitment_bot import PrecommitmentBot

class CDT:
    def __init__(self, logger):
        self.logger = logger

    def name():
        return "cdt"

    def decide(self, scenario, decision_name, sim):
        agent_name, actions = scenario.decision_table[decision_name]

        self.logger.log(f"I, {agent_name}, am deciding {decision_name} using CDT.")

        bots = PrecommitmentBot.all_possible_bots(self.logger, scenario.decision_table)
        prob = Decimal(1.0) / len(bots)
        prior = [
            (prob, bot.decide, bot.decide, bot.description())
            for bot in bots
        ]
        start_event = scenario.events[scenario.start_event]
        def stop(event):
            return ((event.label == "decide" or event.label == "forced_decide")
                and event.decision_name == decision_name)
        distr = sim.simulate_with_distribution(prior, scenario, start_event, stop)

        with self.logger.group(f"Probability distribution of my current event:"):
            for event, prob in distr.items():
                self.logger.log(f"  {event.id} -> {prob}")

        normal_distr = normalize_distribution(distr)
        with self.logger.group(f"Normalized probability distribution of my current event:"):
            for event, prob in normal_distr.items():
                self.logger.log(f"  {event.id} -> {prob}")

        action_to_utility = {}
        with self.logger.group(f"Considering consequences of possible actions:"):
            for action in actions:
                with self.logger.group(f"with action '{action}':"):
                    with self.logger.group(f"Computing utility starting from my current event:"):
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
