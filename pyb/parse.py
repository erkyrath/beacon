#!/usr/bin/env python3

import sys
import re
from enum import StrEnum

pat_white = re.compile('^[ \t]+')
pat_symbol = re.compile('^[a-zA-Z_][a-zA-Z_0-9]*')
pat_number = re.compile('^[-]?[0-9]*[.]?[0-9]+')
pat_color = re.compile('^[$][0-9a-fA-F]+')

def parse(filename):
    fl = open(filename)
    for ln in fl.readlines():
        ln = ln.rstrip()
        match = pat_white.match(ln)
        if not match:
            indent = 0
        else:
            val = match.group(0)
            indent = len(val.replace('\t', '    '))
            cut = len(val)
            ln = ln[ cut : ]
        if not ln or ln.startswith('#'):
            continue
        ls = lex(ln)
        print(indent, ls)

class TokType(StrEnum):
    SYMBOL = 'SYMBOL'
    NUM = 'NUM'
    COLOR = 'COLOR'
    COMMA = 'COMMA'
    EQUALS = 'EQUALS'
    COLON = 'COLON'
    QUOTE = 'QUOTE'

class Token:
    def __init__(self, typ, val=None):
        self.typ = typ
        self.val = val

    def __repr__(self):
        if self.val is None:
            return '<Token %s>' % (self.typ,)
        else:
            return '<Token %s %r>' % (self.typ, self.val,)

def lex(ln):
    res = []
    while ln:
        match = pat_white.match(ln)
        if match:
            val = match.group(0)
            ln = ln[ len(val) : ]
        if not ln:
            break

        if ln.startswith(':'):
            tok = Token(TokType.COLON)
            res.append(tok)
            ln = ln[ 1 : ]
            continue
            
        if ln.startswith(','):
            tok = Token(TokType.COMMA)
            res.append(tok)
            ln = ln[ 1 : ]
            continue
            
        if ln.startswith('='):
            tok = Token(TokType.EQUALS)
            res.append(tok)
            ln = ln[ 1 : ]
            continue
            
        match = pat_symbol.match(ln)
        if match:
            val = match.group(0)
            tok = Token(TokType.SYMBOL, val)
            res.append(tok)
            ln = ln[ len(val) : ]
            continue

        match = pat_number.match(ln)
        if match:
            val = match.group(0)
            fval = float(val)
            tok = Token(TokType.NUM, fval)
            res.append(tok)
            ln = ln[ len(val) : ]
            continue

        match = pat_color.match(ln)
        if match:
            val = match.group(0)
            if len(val) not in [ 4, 7 ]:
                raise Exception('invalid color length: ' + val)
            tok = Token(TokType.COLOR, val)
            res.append(tok)
            ln = ln[ len(val) : ]
            continue

        raise Exception('invalid character: ' + ln)
        
    return res
    
parse('test.pab')

