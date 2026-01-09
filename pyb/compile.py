from enum import StrEnum

from lex import Term, TokType

class Ctx(StrEnum):
    TIME  = 'TIME'
    SPACE = 'SPACE'

class Node:
    classname = '???'
    idcount = 0
    
    def __init__(self):
        self.id = '%s_%d' % (self.classname, Node.idcount,)
        Node.idcount += 1

class NodeConstant:
    classname = 'constant'

    def __init__(self, term, ctx):
        if len(term.args) != 1:
            raise Exception('constant must have one arg')
        arg = term.args[0]
        if arg.tok.typ != TokType.NUM:
            raise Exception('constant must have numeric arg')
        self.value = arg.tok.val
        
def compileall(trees):
    roots = []
    for term in trees:
        root = compile(term, Ctx.SPACE)
        roots.append(( root, term.name ))

    return roots

def compile(term, ctx):
    if term.tok.typ != TokType.SYMBOL:
        raise Exception('non-symbol')
    match term.tok.val:
        case 'constant':
            return NodeConstant(term, ctx)
        case _:
            raise Exception('unknown term id')
