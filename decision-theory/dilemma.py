import xmlschema
import log
import decimal

class Dilemma:
    schema = xmlschema.XMLSchema("dilemma.xsd")
    all_node_types = ["do", "random", "predict", "decide", "outcome"]

    def __init__(self, dilemma_filepath):
        dilemma = Dilemma.schema.decode(dilemma_filepath) # errors on invalid
        #dilemma = Dilemma.__cleanse(dilemma)
        self.name = dilemma["@name"]
        self.author = dilemma["@author"]
        self.agent_names = dilemma["agent"]
        self.dilemma = dilemma
        self.events = {
            event["@name"]: event
            for event in self.dilemma["event"]
        }
        self.start_event = self.dilemma["@start"]

        self.logger = log.Logger()

        # Validation
        if self.start_event not in self.events:
            raise Exception(f"Missing start event: {self.start_event}")

    def show_json(self):
        return pretty.pretty_json(self.dilemma)

    def show_compact_json(self):
        return pretty.pretty_compact_json(self.dilemma)

    def run(self, decision_theories):
        """Run this dilemma with the given decision theory agents"""
        agents = {
            agent_name: theory(agent_name, self.logger)
            for theory, agent_name
            in zip(decision_theories, self.agent_names)
        }
        return self.__run(agents, self.events[self.start_event], self.start_event)

    def __run(self, agents, json, event):
        """
        Run the situation `json` using the given decision procedures for `agents`.
        `event` is the _immediate_ containing event, or `None` if this isn't the
        root node of an event.
        """

        for node in Dilemma.all_node_types:
            if node in json:
                node_type = node
                json = json[node]
                break
        else:
            raise Exception(f"Bug! Parsed as valid, but do not know how to run: {json}")

        if node_type == "do":
            # TODO: cycle detection
            event_name = json["@event"]
            with self.logger.group(f"Event '{event_name}':"):
                result = self.__run(agents, self.events[event_name], event_name)
            return result

        elif node_type == "random":
            with self.logger.group("Random event:"):
                expected_outcome = {}
                for elem in json["case"]:
                    prob = elem["@prob"]
                    with self.logger.group(f"With probability {prob}:"):
                        outcome = self.__run(agents, elem, None)
                        for agent, utility in outcome.items():
                            expected_outcome.setdefault(agent, decimal.Decimal(0.0))
                            expected_outcome[agent] += prob * utility
                with self.logger.group("Average outcome:"):
                    for agent, utility in expected_outcome.items():
                        self.logger.log(f"{agent} -> {utility}")

        elif node_type == "predict":
            agent_name = json["@agent"]
            agent = agents[agent_name]
            theory = agent.__class__.name()
            scenario = json["@in-scenario"]
            decision = json["@making-decision"]
            with self.logger.group(f"Predicting {agent_name}'s decision in scenario '{scenario}' for the decision at '{decision}':"):
                choice = agent.decide(self.events, scenario, decision)
            self.logger.log(f"{agent_name} predicted to choose to {choice}")
            for elem in json["case"]:
                if elem["@choice"] == choice:
                    return self.__run(agents, elem, None)
            self.logger.log(f"{agent_name}'s predicted decision is invalid.")
            return None # Invalid decision: agent explodes.

        elif node_type == "decide":
            if event is None:
                raise Exception("Bad dilemma. To make it easy to refer to choices,"
                  + "'decide' can only be the top-level node in an event. "
                  + "To fix this, add a new event for the decision.")
            agent_name = json["@agent"]
            agent = agents[agent_name]
            with self.logger.group(f"{agent_name} is making a decision:"):
                choice = agent.decide(self.events, self.start_event, event)
            self.logger.log(f"{agent_name} chooses to {choice}")
            for elem in json["case"]:
                if elem["@choice"] == choice:
                    return self.__run(agents, elem, None)
            self.logger.log(f"{agent_name}'s decision is invalid.")
            return None # Invalid decision: agent explodes.

        elif node == "outcome":
            outcome = {
                elem["@agent"]: elem["@amount"]
                for elem in json["utility"]
            }
            with self.logger.group("Outcome:"):
                for agent, utility in outcome.items():
                    self.logger.log(f"{agent} -> {utility}")
            return outcome

        else:
            raise Exception("Bug! Parsed as valid, but do not know how to handle: {json}")

def display_outcome(outcome, logger):
    for agent, utility in outcome.items():
        logger.log(f"{agent}: {utility}")
