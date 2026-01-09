#!/usr/bin/env python3

import sys

from lex import parselines
from compile import compileall

def parse(filename):
    fl = open(filename)
    parsetrees = parselines(fl)
    fl.close()

    compileall(parsetrees)


root = parse(sys.argv[1])
