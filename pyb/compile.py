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

    def __repr__(self):
        return '<%s>' % (self.id,)

class NodeConstant(Node):
    classname = 'constant'

    def __init__(self, term, ctx, asnum=None):
        Node.__init__(self)
        if term is None and asnum is not None:
            self.value = asnum
            return
        if len(term.args) != 1:
            raise Exception('constant must have one arg')
        arg = term.args[0]
        if arg.tok.typ != TokType.NUM:
            raise Exception('constant must have numeric arg')
        self.value = arg.tok.val

class NodeLinear(Node):
    classname = 'linear'

    def __init__(self, term, ctx):
        Node.__init__(self)
        if len(term.args) != 2:
            raise Exception('linear must have two args')
        argstart = term.args[0]
        assert argstart.name == 'start'
        argvel = term.args[1]
        assert argvel.name == 'velocity'
        self.start = compile(argstart, ctx)
        self.velocity = compile(argvel, ctx)
        
def compileall(trees):
    roots = []
    for term in trees:
        root = compile(term, Ctx.SPACE)
        roots.append(( root, term.name ))
        print(root)

    return roots

def compile(term, ctx):
    if term.tok.typ == TokType.NUM:
        return NodeConstant(None, ctx, asnum=term.tok.val)
    if term.tok.typ != TokType.SYMBOL:
        raise Exception('non-symbol')
    match term.tok.val:
        case 'constant':
            return NodeConstant(term, ctx)
        case 'linear':
            return NodeLinear(term, ctx)
        case _:
            raise Exception('unknown term id')
