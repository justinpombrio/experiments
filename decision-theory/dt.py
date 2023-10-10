import pretty
import log
import dilemma
import cdt_agent

# The agent making a choice is provided:
# 1. All events, giving them a world model
# 2. The event name of the situation they started in (necessary to know their past)
# 3. The event name of the decision they're currently making

# TODO: hypothetical depth?
# TODO: timeout?
# TODO: guard against agents running the dilemma

def validate(dilemma_filepath):
    situation = dilemma.Dilemma(dilemma_filepath)
    situation.print()
    print("VALID")

def run(dilemma_filepath, agents):
    situation = dilemma.Dilemma(dilemma_filepath)
    print(f"Running dilemma: {situation.name}")
    print(f"with agents: {[agent.name() for agent in agents]}")
    result = situation.run(agents)
    print("Result:")
    print(result)

run("hitchhiker_dilemma.xml", [cdt_agent.CDT])
