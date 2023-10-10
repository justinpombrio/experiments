import xmlschema
import pretty

# The agent making a choice is provided:
# 1. All events, giving them a world model
# 2. The event name of the situation they started in (necessary to know their past)
# 3. The event name of the decision they're currently making

# TODO: hypothetical depth?
# TODO: timeout?

TAB = "    "
def indent(indent_level):
    return TAB * indent_level

class Logger:
    def __init__(self):
        self.indent_level = 0

    def block(self):
        logger = self
        class ContextManager:
            def __enter__(self):
                logger.indent_level += 1
            def __exit__(self):
                logger.indent_level -= 1
                if logger.indent_level < 0:
                    raise Exception("Logger: too many 'close()'s")
        return ContextManager
    
    def log(self, message):
        print(f"{indent(self.indent_level)}{message}")

def make_logger(prefix):
    return lambda msg: print(prefix + ": " + msg)

class Dilemma:
    schema = xmlschema.XMLSchema("dilemma.xsd")
    all_node_types = ["do", "predict", "decide", "outcome"]

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

        # TODO:
        self.log = print

        # Validation
        if self.start_event not in self.events:
            raise Exception(f"Missing start event: {self.start_event}")

        print(pretty.pretty_json(self.dilemma))
        for event in self.events.values():
            print(event)
            print(pretty.pretty_json(event))
            print(pretty.pretty_compact_json(event))

    def print(self):
        print(pretty.pretty_compact_json(self.dilemma))

    def to_json(self):
        return pretty.pretty_json(self.dilemma)

    def run(self, decision_theories):
        """Run this dilemma with the given decision theory agents"""
        agents = {
            agent_name: theory(agent_name, make_logger(theory.name()))
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
            raise Exception(f"Bug! Parsed as valid, but do not know how to handle: {json}")

        if node_type == "do":
            # TODO: cycle detection
            return self.__run(agents, self.events[json["@event"]], event)

        elif node_type == "predict":
            agent = agents[json["@agent"]]
            theory = agent.__class__.name()
            scenario = json["@in-scenario"]
            decision = json["@making-decision"]
            self.log(f"Predicting {theory} agent's decision in scenario '{scenario}' for the decision at '{decision}'")
            choice = agent.decide(self.events, scenario, decision)
            self.log(f"Agent predicted to choose to {choice}")
            for elem in json["case"]:
                if elem["@choice"] == choice:
                    return self.__run(agents, elem, None)
            self.log(f"Agent's predicted decision is invalid.")
            return None # Invalid decision: agent explodes.

        elif node_type == "decide":
            if event is None:
                raise Exception("Bad dilemma. To make it easy to refer to choices,"
                  + "'decide' can only be the top-level node in an event. "
                  + "To fix this, add a new event for the decision.")
            agent = agents[json["@agent"]]
            choice = agent.decide(
                self.events,
                self.start_event,
                event
            )
            for elem in json["case"]:
                if elem["@choice"] == choice:
                    return self.__run(agents, elem)
            return None # Invalid decision: agent explodes.

        elif node == "outcome":
            return {
                elem["@agent"]: elem["@amount"]
                for elem in json["utility"]
            }

        else:
            raise Exception("Bug! Parsed as valid, but do not know how to handle: {json}")

    # TODO: delete
    def __find(self, event_name):
        for event in self.events:
            if event["@name"] == event_name:
                return event
        raise Exception(f"Bad dilemma: event name {event_name} not found.")

class CDT:
    def __init__(self, agent_name, log=print):
        self.agent_name = agent_name
        self.log = log

    def name():
        return "CDT"

    def decide(self, events, _situation, decision):
        self.log(f"Considering all my options, starting from '{decision}'...")
        outcomes = {}
        for elem in events[decision]["decide"]["case"]:
            choice = elem["@choice"]
            self.log(f"  Considering what happens if I {choice}:")
            utility = self.expected_utility(events, elem)
            outcomes[choice] = utility

        self.log("My choices, and their utitilies, are:")
        for choice, utility in outcomes.items():
            self.log(f"  {choice} -> {utility}")

        best_choice = pick_best_choice(outcomes)
        best_utility = outcomes[best_choice]
        self.log(f"So my best choice is {best_choice} for a utility of {best_utility}")
        return best_choice

    def expected_utility(self, events, json):
        if "outcome" in json:
            for elem in json["outcome"]["utility"]:
                if elem["@agent"] == self.agent_name:
                    amount = elem["@amount"]
                    self.log(f"    Expected utility of outcome: {amount}")
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

def validate(dilemma_filepath):
    dilemma = Dilemma(dilemma_filepath)
    dilemma.print()
    print("VALID")

def run(dilemma_filepath, agents):
    dilemma = Dilemma(dilemma_filepath)
    print(f"Running dilemma: {dilemma.name}")
    print(f"with agents: {[agent.name() for agent in agents]}")
    result = dilemma.run(agents)
    print("Result:")
    print(result)

run("hitchhiker-dilemma.xml", [CDT])
