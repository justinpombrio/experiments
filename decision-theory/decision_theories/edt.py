from decimal import Decimal
from decision_theories.precommitment_bot import PrecommitmentBot

class EDT:
    def __init__(self, logger):
        self.logger = logger

    def name():
        return "edt"

    def decide(self, scenario, decision_name, sim):
        agent_name, actions = scenario.decision_table[decision_name]

        self.logger.log(f"I, {agent_name}, am deciding {decision_name} using EDT.")

        bot_to_distr = {}
        with self.logger.group(f"Using prior to compute distribution over current event:"):
            bots = PrecommitmentBot.all_possible_bots(self.logger, scenario.decision_table)
            prob = Decimal(1.0) / len(bots)
            start_event = scenario.events[scenario.start_event]
            def stop(event):
                return ((event.label == "decide" or event.label == "forced_decide")
                    and event.decision_name == decision_name)
            for bot in bots:
                with self.logger.group(f"with precommitment {bot.description()}:"):
                    distr = sim.simulate(bot.decide, bot.decide, scenario, start_event, stop)
                    bot_to_distr[bot] = distr

        action_to_bot_to_distr = {}
        def add_case(action, bot, event, prob):
            action_to_bot_to_distr.setdefault(action, {})
            action_to_bot_to_distr[action].setdefault(bot, {})
            action_to_bot_to_distr[action][bot].setdefault(event, Decimal(0.0))
            action_to_bot_to_distr[action][bot][event] += prob

        for bot, distr in bot_to_distr.items():
            for event, prob in distr.items():
                if event.label == "decide":
                    action = bot.precommitments[agent_name, decision_name]
                    add_case(action, bot, event.cases[action], prob)
                elif event.label == "forced_decide":
                    add_case(event.action, bot, event.case, prob)

        with self.logger.group(f"Probability distribution over current event:"):
            for action, bot_to_distr in action_to_bot_to_distr.items():
                with self.logger.group(f"for action {action}:"):
                    for bot, distr in bot_to_distr.items():
                        with self.logger.group(f"with precommitment {bot.description()}:"):
                            for event, prob in distr.items():
                                self.logger.log(f"{event.id} -> {prob}")

        action_to_bot_to_normalized_distr = {}
        for action, bot_to_distr in action_to_bot_to_distr.items():
            total_prob = Decimal(0.0)
            for bot, distr in bot_to_distr.items():
                for _event, prob in distr.items():
                    total_prob += prob
            bot_to_normalized_distr = {
                bot : {
                    event : prob / total_prob
                    for event, prob in distr.items()
                }
                for bot, distr in bot_to_distr.items()
            }
            action_to_bot_to_normalized_distr[action] = bot_to_normalized_distr

        with self.logger.group(f"Normalized probability distribution over current event:"):
            for action, bot_to_distr in action_to_bot_to_normalized_distr.items():
                with self.logger.group(f"for action {action}:"):
                    for bot, distr in bot_to_distr.items():
                        with self.logger.group(f"with precommitment {bot.description()}:"):
                            for event, prob in distr.items():
                                self.logger.log(f"{event.id} -> {prob}")
        
        action_to_utility = {}
        with self.logger.group(f"Computing expected utility given each action:"):
            for action, bot_to_distr in action_to_bot_to_normalized_distr.items():
                for bot, distr in bot_to_distr.items():
                    for event, prob in distr.items():
                        outcome = sim.simulate(bot.decide, bot.decide, scenario, event)
                        action_to_utility.setdefault(action, Decimal(0.0))
                        action_to_utility[action] += outcome[agent_name] * prob

        with self.logger.group(f"Expected utility for each action:"):
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
