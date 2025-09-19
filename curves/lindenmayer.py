# Input: The 'string' in the Lindenmayer rule 'P -> string',
#        using the syntax from src/curve.rs.
# Output:
#   - P -> string           -- the input
#   - Q -> reflect(string)
#   - R -> reverse(string)
#   - S -> reflect(reverse(string)) = reverse(reflect(string))

import sys

REVERSE_MAP_SPEC = "ff,+-,PQ,RS"
REFLECT_MAP_SPEC = "ff,+-,PR,QS"

def spec_to_map(spec):
    mapping = {}
    for pair in spec.split(","):
        mapping[pair[0]] = pair[1]
        mapping[pair[1]] = pair[0]
    return mapping

REVERSE_MAP = spec_to_map(REVERSE_MAP_SPEC)
REFLECT_MAP = spec_to_map(REFLECT_MAP_SPEC)

def reverse(string):
    """Compute the reverse of a Lindenmayer string (do it backwards)"""
    return "".join([REVERSE_MAP[ch] for ch in reversed(string)])

def reflect(string):
    """Compute the left-right reflection of a Lindenmayer string (reflect across
    a vertical line)"""
    return "".join([REFLECT_MAP[ch] for ch in string])

string = sys.argv[1]
print("P", "->", string)
print("Q", "->", reflect(string))
print("R", "->", reverse(string))
print("S", "->", reflect(reverse(string)))
