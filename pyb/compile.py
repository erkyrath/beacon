from enum import StrEnum
from collections import namedtuple

from lex import Term, TokType

class Ctx(StrEnum):
    TIME  = 'TIME'
    SPACE = 'SPACE'

class WaveShape(StrEnum):
    FLAT = 'FLAT'
    SQUARE = 'SQUARE'
    HALFSQUARE = 'HALFSQUARE'
    TRIANGLE = 'TRIANGLE'
    TRAPEZOID = 'TRAPEZOID'
    SAWTOOTH = 'SAWTOOTH'
    SQRTOOTH = 'SQRTOOTH'
    SAWDECAY = 'SAWDECAY'
    SQRDECAY = 'SQRDECAY'
    SINE = 'SINE'
    
class ArgFormat:
    def __init__(self, name, typ, anon=False, multiple=False, default=None):
        self.name = name
        self.typ = typ
        self.anon = anon
        self.multiple = multiple
        self.default = default
        self.isoptional = (default is not None)

    def __repr__(self):
        defaultstr = ' = %s' if self.default else ''
        return '<ArgFormat "%s" %s%s>' % (self.name, self.typ, defaultstr,)
    
class Node:
    classname = '???'
    idcount = 0

    argformat = None
    argformatmap = None
    argclass = None

    usesimplicit = False

    allclassmap = {}

    @staticmethod
    def prepclasses(classls):
        for cla in classls:
            Node.allclassmap[cla.classname] = cla
            cla.argformatmap = dict([(argf.name, argf) for argf in cla.argformat])
            cla.argclass = namedtuple('Args_'+cla.classname, [ argf.name for argf in cla.argformat ])
    
    def __init__(self, ctx):
        self.id = '%s_%d' % (self.classname, Node.idcount,)
        Node.idcount += 1

        self.implicit = ctx
        self.depend = AxisDep.NONE
        self.buffered = False

    def __repr__(self):
        return '<%s>' % (self.id,)

    def parseargs(self, args):
        map = {}
        pos = 0
        for arg in args:
            if arg.name:
                argf = self.argformatmap[arg.name]
            else:
                argf = self.argformat[pos]
                pos += 1
            if argf.name in map:
                ### multiple?
                raise Exception('%s: duplicate arg %s' % (self.classname, argf.name))
            if argf.typ is float:
                if arg.tok.typ is not TokType.NUM:
                    raise Exception('%s: %s must be numeric' % (self.classname, argf.name))
                map[argf.name] = arg.tok.val
            elif argf.typ is WaveShape:
                if arg.tok.typ is not TokType.SYMBOL:
                    raise Exception('%s: unrecognized waveshape' % (argf.name,))
                map[argf.name] = WaveShape.__members__[arg.tok.val.upper()]
            elif argf.typ is Node:
                map[argf.name] = compile(arg, self.implicit)
            elif argf.typ is Ctx.TIME:
                map[argf.name] = compile(arg, Ctx.TIME)
            elif argf.typ is Ctx.SPACE:
                map[argf.name] = compile(arg, Ctx.SPACE)
            else:
                raise Exception('%s: unimplemented arg type: %s' % (self.classname, argf.name))

        for argf in self.argformat:
            if argf.name not in map and argf.isoptional:
                if not (isinstance(argf.default, int) or isinstance(argf.default, float)):
                    raise Exception('%s: arg default is not numeric: %s' % (self.classname, argf.name))
                map[argf.name] = NodeConstant(Ctx.TIME, asnum=argf.default)

        self.args = self.argclass(**map)

    def getarg(self, key):
        return getattr(self.args, key)

    def generateimplicit(self):
        if not self.usesimplicit:
            raise Exception('usesimplicit not set')
        if self.implicit is Ctx.TIME:
            ### relative to start?
            return "clock"
        if self.implicit is Ctx.SPACE:
            return "(ix/pixelCount)"
        raise Exception('implicit not set')

    def dump(self, indent=0, name=None):
        indentstr = '  '*indent
        namestr = name+'=' if name else ''
        impstr = str(self.implicit)[0]
        depstr = axisdepname(self.depend)
        print('%s%s<%s> (%s) dep=%s' % (indentstr, namestr, self.id, impstr, depstr))
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

    def __init__(self, ctx, asnum=None):
        Node.__init__(self, ctx)
        if asnum is not None:
            self.args = self.argclass(value=asnum)

    def generatedata(self, ctx):
        return str(self.args.value)

class NodeLinear(Node):
    classname = 'linear'

    usesimplicit = True
    argformat = [
        ArgFormat('start', Ctx.TIME),
        ArgFormat('velocity', Ctx.TIME),
    ]

    def generatedata(self, ctx):
        param = self.generateimplicit()
        startdata = self.args.start.generatedata(ctx=ctx)
        veldata = self.args.velocity.generatedata(ctx=ctx)
        return '(%s + %s * %s)' % (startdata, param, veldata,)

class NodeClamp(Node):
    classname = 'clamp'
    
    usesimplicit = False
    argformat = [
        ArgFormat('arg', Node),
        ArgFormat('min', Ctx.TIME, default=0),
        ArgFormat('max', Ctx.TIME, default=1),
    ]

    def generatedata(self, ctx):
        argdata = self.args.arg.generatedata(ctx=ctx)
        mindata = self.args.min.generatedata(ctx=ctx)
        maxdata = self.args.max.generatedata(ctx=ctx)
        return 'clamp(%s, %s, %s)' % (argdata, mindata, maxdata,)

class NodeWave(Node):
    classname = 'wave'

    usesimplicit = True
    argformat = [
        ArgFormat('shape', WaveShape),
        ArgFormat('min', Ctx.TIME, default=0),
        ArgFormat('max', Ctx.TIME, default=1),
        ArgFormat('period', Ctx.TIME, default=1),
        ### offset?
    ]

    def generatedata(self, ctx):
        param = self.generateimplicit()
        mindata = self.args.min.generatedata(ctx=ctx)
        maxdata = self.args.max.generatedata(ctx=ctx)
        perioddata = self.args.period.generatedata(ctx=ctx)
        match self.args.shape:
            case WaveShape.SINE:
                return '(%s+%s*(0.5-0.5*cos(PI2*%s/%s)))' % (mindata, '(%s-%s)'%(maxdata,mindata,), param, perioddata)
            case _:
                raise Exception('unimplemented WaveShape')

nodeclasses = [
    NodeConstant,
    NodeLinear,
    NodeClamp,
    NodeWave,
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
        return NodeConstant(ctx, asnum=term.tok.val)
    if term.tok.typ != TokType.SYMBOL:
        raise Exception('non-symbol')
    cla = Node.allclassmap.get(term.tok.val.lower())
    if not cla:
        raise Exception('unknown term: %s' % (term.tok.val,))
    nod = cla(ctx)
    nod.parseargs(term.args)
    return nod

# Late imports
from program import Program, AxisDep, axisdepname
