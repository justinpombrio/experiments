# External dependencies
import argparse

# Local modules
from pretty import pretty_json, pretty_compact_json
from log import Logger
from dilemma import Dilemma
from simulate import Simulator
from decision_theories.cdt import CDT
from decision_theories.edt import EDT
from decision_theories.epdt import EPDT
from decision_theories.udt import UDT

DECISION_THEORY_LIST = [CDT, EDT, EPDT, UDT]
DECISION_THEORIES = {
    dt.name() : dt
    for dt in DECISION_THEORY_LIST
}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        prog = "dts",
        description = "Decision Theory Simulator"
    )
    subparsers = parser.add_subparsers(dest="mode", required=True)

    parser_validate = subparsers.add_parser("validate",
        help="Validate and print an dilemma XML file")
    parser_validate.add_argument("dilemma_filename",
        help="Path to a .xml file obeying the spec 'dilemma.xsd'")
    parser_validate.add_argument("--compact", action="store_true",
        help="Print the dilemma in a prettier, more compact format")

    parser_run = subparsers.add_parser("run",
        help="Run an dilemma XML file with a set of decision theoretic agents")
    parser_run.add_argument("dilemma_filename",
        help="Path to a .xml file obeying the spec 'dilemma.xsd'")
    parser_run.add_argument("decision_theory",
        help="The name of the decision theory to be used by the agents.")

    args = parser.parse_args()
    if args.mode == "validate":
        dilemma = Dilemma(args.dilemma_filename)
        if args.compact:
            print(pretty_compact_json(dilemma.json))
        else:
            print(pretty_json(dilemma.json))
        print("VALID")

    elif args.mode == "run":
        dilemma = Dilemma(args.dilemma_filename)
        if args.decision_theory not in DECISION_THEORIES:
            raise Exception(f"Unknown decision theory: '{args.decision_theory}'")
        logger = Logger()
        theory = DECISION_THEORIES[args.decision_theory](logger)
        decide = theory.decide
        predict = theory.decide # predictors are accurate
        sim = Simulator(logger)
        start_event = dilemma.scenario.events[dilemma.scenario.start_event]
        outcome = sim.simulate(decide, predict, dilemma.scenario, start_event)
        with logger.group(f"Final outcome:"):
            for agent, utility in outcome.items():
                logger.log(f"{agent} -> {utility:,}")
