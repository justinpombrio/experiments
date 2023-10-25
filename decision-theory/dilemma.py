import xmlschema

class Goto:
    def __init__(self, json):
        self.label = "goto"
        self.id = json["@id"]
        self.event_name = json["@event"]

class Random:
    def __init__(self, json):
        self.label = "random"
        self.id = json["@id"]
        self.cases = [
            (case["@prob"], parse_event(case))
            for case in json["case"]
        ]

class Predict:
    def __init__(self, json):
        self.label = "predict"
        self.id = json["@id"]
        self.agent_name = json["@agent"]
        self.decision_name = json["@decision"]
        self.start_event = json["scenario"]["@start"]
        self.agent_names = json["scenario"]["agent"]
        self.event_names = json["scenario"]["event"]
        self.cases = {
            case["@action"]: parse_event(case)
            for case in json["case"]
        }

class Decide:
    def __init__(self, json):
        self.label = "decide"
        self.id = json["@id"]
        self.agent_name = json["@agent"]
        self.decision_name = json["@decision"]
        self.cases = {
            case["@action"]: parse_event(case)
            for case in json["case"]
        }

class ForcedDecide:
    def __init__(self, json):
        self.label = "forced_decide"
        self.id = json["@id"]
        self.agent_name = json["@agent"]
        self.decision_name = json["@decision"]
        self.action = json["case"]["@action"]
        self.case = parse_event(json["case"])

class Outcome:
    def __init__(self, json):
        self.label = "outcome"
        self.id = json["@id"]
        self.utilities = {
            elem["@agent"]: elem["@amount"]
            for elem in json["utility"]
        }

EVENT_TYPES = {
    "goto": Goto,
    "random": Random,
    "predict": Predict,
    "decide": Decide,
    "forced_decide": ForcedDecide,
    "outcome": Outcome
}

def parse_event(json):
    for event_type in EVENT_TYPES:
        if event_type in json:
            return EVENT_TYPES[event_type](json[event_type])
    else:
        raise Exception(f"Bug! Parsed as valid, but do not know how to handle: {json}")

class Scenario:
    def __init__(self, agent_names, events, start_event):
        """
        agent_names: list of agent names
        events: map from event name to Event
        start_event: name of intitial event

        decision_table: {decision_name : {agent_name, list[action]}}
        """
        self.agent_names = agent_names
        self.events = events
        self.start_event = start_event

        start_event = self.events[self.start_event]
        self.decision_table = {}
        self.__make_decision_table(self.decision_table)

    def __make_decision_table(self, table, event=None):
        """
        Returns { decision_name : (agent_name, list[action]) }
        """
        if event is None:
            for _event_name, event in self.events.items():
                self.__make_decision_table(table, event)
            return
        
        if event.label == "goto":
            return
        elif event.label == "random":
            for _prob, case in event.cases:
                self.__make_decision_table(table, case)
        elif event.label == "predict":
            for case in event.cases.values():
                self.__make_decision_table(table, case)
        elif event.label == "decide":
            agent_name = event.agent_name
            decision_name = event.decision_name
            actions = sorted(list(event.cases.keys()))
            if decision_name in table:
                existing_agent_name, existing_actions = table[decision_name]
                if existing_agent_name != agent_name:
                    raise Exception(f"Scenario: inconsistent decision '{decision_name}'. Sometimes made by {existing_agent_name}; sometimes by {agent_name}.")
                if existing_actions != actions:
                    raise Exception(f"Scenario: inconsistent decision '{decision_name}'. Sometimes allows actions {existing_actions}; sometimes {actions}.")
            else:
                table[decision_name] = agent_name, actions
            for case in event.cases.values():
                self.__make_decision_table(table, case)
        elif event.label == "forced_decide":
            pass # TODO: check, but it's annoying to do in arbitrary order
        elif event.label == "outcome":
            pass
        else:
            raise Exception(f"Scenario: unrecognized event label: '{event.label}'")

    def paths_to_decision(self, event, decision_name):
        """
        Find all paths from the given start `event` to the decision named `decision_name`.
        Each path is a sequence of pairs, which have one of the forms:
        - (event:Random, prob:float)
        - (event:Decide, None)       // if has name `decision_name`
        - (event:Decide, action:str) // otherwise
        """
        if event.label == "goto":
            return self.paths_to_decision(event.event_name, decision_name)
        elif event.label == "random":
            decisions = []
            for prob, case in event.cases.items():
                for decision in self.paths_to_decision(case, decision_name):
                    decisions.append([(event, prob)] + decision)
            return decisions
        elif event.label == "predict":
            decisions = []
            for action, case in event.cases.items():
                for decision in self.paths_to_decision(case, decision_name):
                    decisions.append([(event, action)] + decision)
            return decisions
        elif event.label == "decide":
            if event.decision_name == decision_name:
                return [[(event, None)]]
            decisions = []
            for action, case in event.cases.items():
                for decision in self.paths_to_decision(case, decision_name):
                    decisions.append([(event, action)] + decision)
            return decisions
        elif event.label == "outcome":
            return []
        else:
            raise Exception(f"Scenario: unrecognized event label: '{event.label}'")

class Dilemma:
    schema = xmlschema.XMLSchema("dilemma.xsd")

    def __init__(self, dilemma_filepath):
        dilemma = Dilemma.schema.decode(dilemma_filepath) # errors on invalid
        self.name = dilemma["@name"]
        self.xml_author = dilemma["@xml-author"]
        self.description = dilemma["description"]
        self.json = dilemma

        agent_names = dilemma["scenario"]["agent"]
        events = {
            next(iter(event.values()))["@id"]: parse_event(event)
            for event in dilemma["scenario"]["event"]
        }
        start_event = dilemma["scenario"]["@start"]
        self.scenario = Scenario(agent_names, events, start_event)

        # Validation
        if start_event not in events:
            raise Exception(f"Missing start event: {self.start_event}")
