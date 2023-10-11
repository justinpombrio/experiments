class CDT:
    def __init__(self, agent_name, logger):
        self.agent_name = agent_name
        self.logger = logger

    def name():
        return "CDT"

    def decide(self, events, situation, decision):
        with self.logger.group(f"Considering all my options, starting from '{decision}'..."):
            outcomes = {}
            for elem in events[decision]["decide"]["case"]:
                action = elem["@action"]
                with self.logger.group(f"Considering what happens if I {action}:"):
                    utility = self.expected_utility(events, situation, elem)
                outcomes[action] = utility

        with self.logger.group("My possible actions, and their utitilies, are:"):
            for action, utility in outcomes.items():
                self.logger.log(f"  {action} -> {utility}")

        best_action = pick_best_action(outcomes)
        best_utility = outcomes[best_action]
        self.logger.log(f"So my best action is {best_action} for a utility of {best_utility}")
        return best_action

    def expected_utility(self, events, scenario, json):
        if "do" in json:
            json = json["do"]
            event_name = json["@event"]
            with self.logger.group(f"Considering event '{event_name}':"):
                result = self.expected_utility(events, scenario, events[event_name])
            return result

        elif "random" in json:
            json = json["random"]
            expected_utility = 0.0
            with self.logger.group("Considering random event:"):
                for case in json["case"]:
                    prob = case["@prob"]
                    with self.logger.group("With probability {prob}:"):
                        utility = self.expected_utility(events, scenario, case)
                        expected_utility += prob * utility
                self.logger.log("Total expected utility is {expected_utility}")
            return expected_utility

        elif "predict" in json:
            json = json["predict"]
            agent_name = json["@agent"]
            new_scenario = json["@in-scenario"]
            decision = json["@making-decision"]

            agent = self if agent_name == self.agent_name else CDT(agent_name, self.logger)
            who = "myself" if agent_name == self.agent_name else agent_name
            with self.logger.group("Someone will predict {who} making a decision:"):
                action = agent.decide(events, new_scenario, decision)
            self.logger.log("I believe they will predict that {who} chooses to {action}")

            for case in json["case"]:
                if case["@action"] == action:
                    return self.expected_utility(events, scenario, case)
            raise Exception("CDT: invalid action")

        elif "decide" in json:
            json = json["decide"]
            agent_name = json["@agent"]

            agent = self if agent_name == self.agent_name else CDT(agent_name, self.logger)
            who = "I" if agent_name == self.agent_name else agent_name
            with self.logger.group("{who} will make a decision:"):
                action = agent.decide(events, scenario, json["@name"])
            self.logger.log(f"I believe {who} will choose to {action}")

            for case in json["case"]:
                if case["@action"] == action:
                    return self.expected_utility(events, scenario, case)
            raise Exception("CDT: invalid action")

        elif "outcome" in json:
            json = json["outcome"]
            for elem in json["utility"]:
                if elem["@agent"] == self.agent_name:
                    amount = elem["@amount"]
                    self.logger.log(f"Expected utility: {amount}")
                    return amount
            raise Exception("CDT: bad dilemma 1")

        else:
            raise Exception("CDT: non-trivial cases NYI")

def pick_best_action(action_to_utility_map):
    best_action = None
    best_utility = None
    for action, utility in action_to_utility_map.items():
        if best_utility is None or utility > best_utility:
            best_utility = utility
            best_action = action
    return best_action
