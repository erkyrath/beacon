from enum import StrEnum
from collections import namedtuple

from lex import Term, TokType

class Ctx(StrEnum):
    TIME  = 'TIME'
    SPACE = 'SPACE'

class ArgFormat:
    def __init__(self, name, typ, anon=False, multiple=False, default=None):
        self.name = name
        self.typ = typ
        self.anon = anon
        self.multiple = multiple
        self.default = default
        self.optional = (default is not None)

    def __repr__(self):
        defaultstr = ' = %s' if self.default else ''
        return '<ArgFormat "%s" %s%s>' % (self.name, self.typ, defaultstr,)
    
class Node:
    classname = '???'
    idcount = 0

    argformat = None
    argformatmap = None
    argclass = None

    @staticmethod
    def prepclasses(classls):
        for cla in classls:
            cla.argformatmap = dict([(argf.name, argf) for argf in cla.argformat])
            cla.argclass = namedtuple('Args_'+cla.classname, [ argf.name for argf in cla.argformat ])
    
    def __init__(self, ctx):
        self.id = '%s_%d' % (self.classname, Node.idcount,)
        Node.idcount += 1

        self.implicit = ctx
        self.buffered = False

    def __repr__(self):
        return '<%s>' % (self.id,)

    def getarg(self, key):
        return getattr(self.args, key)

    def dump(self, indent=0, name=None):
        indentstr = '  '*indent
        namestr = name+'=' if name else ''
        impstr = str(self.implicit)[0]
        print('%s%s<%s> (%s)' % (indentstr, namestr, self.id, impstr))
        for argf in self.argformat:
            arg = self.getarg(argf.name)
            if not argf.multiple:
                argls = [ arg ]
            else:
                argls = arg
            for arg in argls:
                if isinstance(arg, Node):
                    arg.dump(indent+1, name=argf.name)
                else:
                    print('%s  %s=%s' % (indentstr, argf.name, arg,))
            

class NodeConstant(Node):
    classname = 'constant'

    argformat = [
        ArgFormat('value', float)
    ]

    def __init__(self, term, ctx, asnum=None):
        Node.__init__(self, ctx)
        if term is None and asnum is not None:
            self.args = self.argclass(value=asnum)
            return
        if len(term.args) != 1:
            raise Exception('constant must have one arg')
        arg = term.args[0]
        if arg.tok.typ != TokType.NUM:
            raise Exception('constant must have numeric arg')
        if arg.args:
            raise Exception('number cannot have args')
        self.args = self.argclass(value=arg.tok.val)
        
    def generatedata(self):
        return str(self.args.value)

class NodeLinear(Node):
    classname = 'linear'

    argformat = [
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
        self.args = self.argclass(
            start=compile(argstart, Ctx.TIME),
            velocity=compile(argvel, Ctx.TIME)
        )

    def generatedata(self):
        startdata = self.args.start.generatedata()
        veldata = self.args.velocity.generatedata()
        return '(%s) + (ix/pixelCount) * (%s)' % (startdata, veldata,)

nodeclasses = [
    NodeConstant,
    NodeLinear,
]

Node.prepclasses(nodeclasses)

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

# Late imports
from program import Program
