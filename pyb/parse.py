#!/usr/bin/env python3

import sys

from lex import parselines

def parse(filename):
    fl = open(filename)
    root = parselines(fl)
    fl.close()

    return root


root = parse(sys.argv[1])

for term in root:
    term.dump()
