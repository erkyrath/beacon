from enum import IntEnum

from lex import Term, TokType
from program import Program

class Ctx(IntEnum):
    NONE  = 0
    TIME  = 1
    SPACE = 2
    SPACETIME = 3

class ArgFormat:
    def __init__(self, name, anon=False, multiple=False, default=None):
        self.name = name
        self.anon = anon
        self.multiple = multiple
        self.default = default
        self.optional = (default is not None)
    
class Node:
    classname = '???'
    idcount = 0
    
    def __init__(self, ctx):
        self.id = '%s_%d' % (self.classname, Node.idcount,)
        Node.idcount += 1

        self.implicit = ctx
        self.buffered = False

    def __repr__(self):
        return '<%s>' % (self.id,)

class NodeConstant(Node):
    classname = 'constant'

    argformat = [
        ArgFormat('value', float)
    ]

    def __init__(self, term, ctx, asnum=None):
        Node.__init__(self, ctx)
        if term is None and asnum is not None:
            self.value = asnum
            return
        if len(term.args) != 1:
            raise Exception('constant must have one arg')
        arg = term.args[0]
        if arg.tok.typ != TokType.NUM:
            raise Exception('constant must have numeric arg')
        if arg.args:
            raise Exception('number cannot have args')
        self.value = arg.tok.val
        
    def generatedata(self):
        return str(self.value)

class NodeLinear(Node):
    classname = 'linear'

    argsformat = [
        ArgFormat('start', Ctx.TIME),
        ArgFormat('velocity', Ctx.TIME),
        ### ArgFormat('pos', Ctx.TIME, default=0.5),
        ### ArgFormat('addend', Node, anon=True, multiple=True),
    ]

    def __init__(self, term, ctx):
        Node.__init__(self, ctx)
        if len(term.args) != 2:
            raise Exception('linear must have two args')
        argstart = term.args[0]
        assert argstart.name == 'start'
        argvel = term.args[1]
        assert argvel.name == 'velocity'
        self.start = compile(argstart, Ctx.TIME)
        self.velocity = compile(argvel, Ctx.TIME)

    def generatedata(self):
        startdata = self.start.generatedata()
        veldata = self.velocity.generatedata()
        return '(%s) + (ix/pixelCount) * (%s)' % (startdata, veldata,)
        
def compileall(trees):
    roots = []
    for term in trees:
        root = compile(term, Ctx.SPACE)
        roots.append(( root, term.name ))

    startnod = None
    map = {}
    for (nod, name) in roots:
        if name is None:
            if startnod is not None:
                raise Exception('more than one start')
            startnod = nod
        else:
            if name in map:
                raise Exception('duplicate def')
            map[name] = nod
    return Program(startnod, map)

def compile(term, ctx):
    if term.tok.typ == TokType.NUM:
        if term.args:
            raise Exception('number cannot have args')
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
