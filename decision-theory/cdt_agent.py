
class CDT:
    def __init__(self, agent_name, logger):
        self.agent_name = agent_name
        self.logger = logger

    def name():
        return "CDT"

    def decide(self, events, _situation, decision):
        with self.logger.group(f"Considering all my options, starting from '{decision}'..."):
            outcomes = {}
            for elem in events[decision]["decide"]["case"]:
                choice = elem["@choice"]
                with self.logger.group(f"Considering what happens if I {choice}:"):
                    utility = self.expected_utility(events, elem)
                outcomes[choice] = utility

        with self.logger.group("My choices, and their utitilies, are:"):
            for choice, utility in outcomes.items():
                self.logger.log(f"  {choice} -> {utility}")

        best_choice = pick_best_choice(outcomes)
        best_utility = outcomes[best_choice]
        self.logger.log(f"So my best choice is {best_choice} for a utility of {best_utility}")
        return best_choice

    def expected_utility(self, events, json):
        if "outcome" in json:
            for elem in json["outcome"]["utility"]:
                if elem["@agent"] == self.agent_name:
                    amount = elem["@amount"]
                    self.logger.log(f"Expected utility: {amount}")
                    return amount
            raise Exception("CDT: bad dilemma 1")
        else:
            raise Exception("CDT: non-trivial cases NYI")

def pick_best_choice(choice_to_utility_map):
    best_choice = None
    best_utility = None
    for choice, utility in choice_to_utility_map.items():
        if best_utility is None or utility > best_utility:
            best_utility = utility
            best_choice = choice
    return best_choice
