TAB = "    "
def indent(indent_level):
    return TAB * indent_level

def pretty_json(json, indent_level = 0):
    """Pretty print json"""

    if type(json) is dict:
        s = "{"
        for i, key in enumerate(json):
            val = pretty_json(json[key], indent_level + 1)
            s += f"\n{indent(indent_level + 1)}\"{key}\": {val}"
            if i != len(json) - 1:
                s += ","
        s += f"\n{indent(indent_level)}}}"
        return s

    elif type(json) is list:
        s = "["
        for i, elem in enumerate(json):
            elem = pretty_json(elem, indent_level + 1)
            s += f"\n{indent(indent_level + 1)}{elem}"
            if i != len(json) - 1:
                s += ","
        s += f"\n{indent(indent_level)}}}"
        return s

    else:
        return str(json)

def pretty_compact_json(json, indent_level=0):
    """Pretty print json in a compact format"""

    if type(json) is dict:
        s = ""
        for key in json:
            val = pretty_compact_json(json[key], indent_level + 1)
            s += f"\n{indent(indent_level)}{key}: {val}"
        return s

    elif type(json) is list:
        s = ""
        for i, elem in enumerate(json):
            numeral = "(" + str(i + 1) + ")"
            elem = pretty_compact_json(elem, indent_level + 1)
            s += f"\n{indent(indent_level)}{numeral} {elem}"
        return s

    else:
        return str(json)
