from enum import StrEnum
from collections import namedtuple

from lex import Term, TokType

class Implicit(StrEnum):
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

    def parseargs(self, args, defmap):
        map = {}
        for arg in args:
            if arg.name:
                argf = self.argformatmap[arg.name]
            else:
                pos = 0
                lastmultiple = None
                while pos < len(self.argformat):
                    argf = self.argformat[pos]
                    if argf.name not in map:
                        break
                    if argf.multiple:
                        lastmultiple = pos
                    pos += 1
                if pos >= len(self.argformat):
                    if lastmultiple is None:
                        raise Exception('%s: too many arguments' % (self.classname,))
                    pos = lastmultiple
                argf = self.argformat[pos]
            if not argf.multiple and argf.name in map:
                raise Exception('%s: duplicate arg: %s' % (self.classname, argf.name))

            argval = None
            
            if argf.typ is float:
                if arg.tok.typ is not TokType.NUM:
                    raise Exception('%s: %s must be numeric' % (self.classname, argf.name))
                argval = arg.tok.val
            elif argf.typ is int:
                if arg.tok.typ is not TokType.NUM:
                    raise Exception('%s: %s must be numeric' % (self.classname, argf.name))
                argval = int(arg.tok.val)
            elif argf.typ is WaveShape:
                if arg.tok.typ is not TokType.SYMBOL:
                    raise Exception('%s: unrecognized waveshape' % (argf.name,))
                argval = WaveShape.__members__[arg.tok.val.upper()]
            elif argf.typ is Node:
                argval = compile(arg, implicit=self.implicit, defmap=defmap)
            elif argf.typ is Implicit.TIME:
                argval = compile(arg, implicit=Implicit.TIME, defmap=defmap)
            elif argf.typ is Implicit.SPACE:
                argval = compile(arg, implicit=Implicit.SPACE, defmap=defmap)
            else:
                raise Exception('%s: unimplemented arg type: %s (%s)' % (self.classname, argf.typ, argf.name))

            if not argf.multiple:
                map[argf.name] = argval
            else:
                if argf.name not in map:
                    map[argf.name] = []
                map[argf.name].append(argval)

        for argf in self.argformat:
            if argf.name not in map and argf.isoptional:
                if not (isinstance(argf.default, int) or isinstance(argf.default, float)):
                    raise Exception('%s: arg default is not numeric: %s' % (self.classname, argf.name))
                map[argf.name] = NodeConstant(Implicit.TIME, asnum=argf.default)

        self.args = self.argclass(**map)

    def getarg(self, key):
        return getattr(self.args, key)

    def getargls(self, key, multiple=False):
        arg = getattr(self.args, key)
        if not multiple:
            return [ arg ]
        else:
            return arg

    def constantval(self):
        return None
        
    def printstaticvars(self):
        pass
    
    def generateimplicit(self):
        if not self.usesimplicit:
            raise Exception('usesimplicit not set')
        if self.implicit is Implicit.TIME:
            ### relative to start?
            return "clock"
        if self.implicit is Implicit.SPACE:
            return "(ix/pixelCount)"
        raise Exception('implicit not set')

    def generatedata(self, ctx):
        if self.buffered:
            if not (self.depend & AxisDep.SPACE):
                return '%s_scalar' % (self.id,)
            else:
                return '%s_pixels[ix]' % (self.id,)
        return self.generateexpr(ctx)

    def generateexpr(self, ctx):
        raise NotImplementedError('generateexpr: %s' % (self.classname,))
    
    def dump(self, indent=0, name=None):
        indentstr = '  '*indent
        namestr = name+'=' if name else ''
        impstr = str(self.implicit)[0]
        depstr = axisdepname(self.depend)
        bufstr = ' (BUF)' if self.buffered else ''
        print('%s%s<%s> (%s) dep=%s%s' % (indentstr, namestr, self.id, impstr, depstr, bufstr))
        for argf in self.argformat:
            argls = self.getargls(argf.name, argf.multiple)
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

    def constantval(self):
        return self.args.value
        
    def generateexpr(self, ctx):
        return str(self.args.value)

class NodeTime(Node):
    classname = 'time'

    usesimplicit = False
    argformat = [
        ArgFormat('arg', Implicit.TIME),
    ]

    def generateexpr(self, ctx):
        argdata = self.args.arg.generatedata(ctx=ctx)
        return argdata

class NodeSpace(Node):
    classname = 'space'

    usesimplicit = False
    argformat = [
        ArgFormat('arg', Implicit.SPACE),
    ]

    def generateexpr(self, ctx):
        argdata = self.args.arg.generatedata(ctx=ctx)
        return argdata

class NodeLinear(Node):
    classname = 'linear'

    usesimplicit = True
    argformat = [
        ArgFormat('start', Implicit.TIME),
        ArgFormat('velocity', Implicit.TIME),
    ]

    def generateexpr(self, ctx):
        param = self.generateimplicit()
        startdata = self.args.start.generatedata(ctx=ctx)
        veldata = self.args.velocity.generatedata(ctx=ctx)
        return '(%s + %s * %s)' % (startdata, param, veldata,)

class NodeClamp(Node):
    classname = 'clamp'
    
    usesimplicit = False
    argformat = [
        ArgFormat('arg', Node),
        ArgFormat('min', Implicit.TIME, default=0),
        ArgFormat('max', Implicit.TIME, default=1),
    ]

    def generateexpr(self, ctx):
        argdata = self.args.arg.generatedata(ctx=ctx)
        mindata = self.args.min.generatedata(ctx=ctx)
        maxdata = self.args.max.generatedata(ctx=ctx)
        return 'clamp(%s, %s, %s)' % (argdata, mindata, maxdata,)

class NodeSum(Node):
    classname = 'sum'
    
    usesimplicit = False
    argformat = [
        ArgFormat('arg', Node, multiple=True),
    ]

    ### constantval if needed...

    def generateexpr(self, ctx):
        argdata = []
        for arg in self.args.arg:
            argdata.append(arg.generatedata(ctx=ctx))
        if len(argdata) == 1:
            return argdata[0]
        return '(%s)' % (' + '.join(argdata),)
    
class NodeMean(Node):
    classname = 'mean'
    
    usesimplicit = False
    argformat = [
        ArgFormat('arg', Node, multiple=True),
    ]

    def generateexpr(self, ctx):
        argdata = []
        for arg in self.args.arg:
            argdata.append(arg.generatedata(ctx=ctx))
        if len(argdata) == 1:
            return argdata[0]
        return '(%s) / %s' % (' + '.join(argdata), len(argdata),)
    
class NodeWave(Node):
    classname = 'wave'

    usesimplicit = True
    argformat = [
        ArgFormat('shape', WaveShape),
        ArgFormat('min', Implicit.TIME, default=0),
        ArgFormat('max', Implicit.TIME, default=1),
        ArgFormat('period', Implicit.TIME, default=1),
        ### offset?
    ]

    def generateexpr(self, ctx):
        param = self.generateimplicit()
        mindata = self.args.min.generatedata(ctx=ctx)
        maxdata = self.args.max.generatedata(ctx=ctx)
        perioddata = self.args.period.generatedata(ctx=ctx)
        if self.implicit is Implicit.SPACE:
            theta = '((%s-0.5)/%s+0.5)' % (param, perioddata,)
        else:
            theta = '%s/%s' % (param, perioddata,)
            
        match self.args.shape:
            case WaveShape.FLAT:
                return maxdata
            case WaveShape.SAWTOOTH:
                minval = ctx.store_val(self, 'min', mindata)
                diffval = ctx.store_val(self, 'diff', '(%s-%s)' % (maxdata, minval,))
                return '(%s+%s*(mod(%s, 1)))' % (minval, diffval, theta)
            case WaveShape.SAWDECAY:
                minval = ctx.store_val(self, 'min', mindata)
                diffval = ctx.store_val(self, 'diff', '(%s-%s)' % (maxdata, minval,))
                return '(%s+%s*(1-mod(%s, 1)))' % (minval, diffval, theta)
            case WaveShape.SQRTOOTH:
                minval = ctx.store_val(self, 'min', mindata)
                diffval = ctx.store_val(self, 'diff', '(%s-%s)' % (maxdata, minval,))
                return '(%s+%s*(pow(mod(%s, 1), 2)))' % (minval, diffval, theta)
            case WaveShape.SQRDECAY:
                minval = ctx.store_val(self, 'min', mindata)
                diffval = ctx.store_val(self, 'diff', '(%s-%s)' % (maxdata, minval,))
                return '(%s+%s*(pow(1-mod(%s, 1), 2)))' % (minval, diffval, theta)
            case WaveShape.TRIANGLE:
                minval = ctx.store_val(self, 'min', mindata)
                diffval = ctx.store_val(self, 'diff', '(%s-%s)' % (maxdata, minval,))
                return '(%s+%s*(triangle(%s)))' % (minval, diffval, theta)
            case WaveShape.HALFSQUARE:
                minval = ctx.store_val(self, 'min', mindata)
                diffval = ctx.store_val(self, 'diff', '(%s-%s)' % (maxdata, minval,))
                return '(%s+%s*(square(%s, 0.5)))' % (minval, diffval, theta)
            case WaveShape.SINE:
                minval = ctx.store_val(self, 'min', mindata)
                hdiffval = ctx.store_val(self, 'hdiff', '((%s-%s)*0.5)' % (maxdata, minval,))
                return '(%s+%s*(1-cos(PI2*%s)))' % (minval, hdiffval, theta)
            case _:
                raise Exception('unimplemented WaveShape')

class NodePulser(Node):
    classname = 'pulser'
    
    usesimplicit = False
    argformat = [
        ArgFormat('maxcount', int),
        ArgFormat('spaceshape', WaveShape, default=WaveShape.TRIANGLE),
        ArgFormat('timeshape', WaveShape, default=WaveShape.SQRDECAY),
        ArgFormat('interval', Implicit.TIME, default=1),
        ArgFormat('pos', Implicit.TIME, default=0.5),
        ArgFormat('duration', Implicit.TIME, default=1),
        ArgFormat('width', Implicit.TIME, default=0.5),
    ]

    def printstaticvars(self):
        maxcount = self.args.maxcount
        print('var %s_live = array(%d)' % (self.id, maxcount,))
        print('var %s_livecount = 0' % (self.id,))
    
    def generateexpr(self, ctx):
        # This is just the initial buffer-clear.
        durationdata = self.args.duration.generatedata(ctx=ctx)
        self.durationdata = durationdata ###
        return '0'

    def pulserprint(self):
        assert self.buffered
        maxcount = self.args.maxcount
        print('  for (var px=0; px<%d; px++) {' % (maxcount,))
        print('    if (%s_live[px]) {' % (self.id,))
        if self.args.timeshape is WaveShape.FLAT:
            print('      timeval = 1')
        else:
            ### calc age
            print('      if (age > 1.0) {\n        %s_live[px] = 0\n        continue\n      }' % (self.id,))
            print('      timeval = triangle(age / %s)' % (self.durationdata,))
            ### timeval = sample timeshape(...)
        print('    }')
        print('  }')
        

nodeclasses = [
    NodeConstant,
    NodeTime,
    NodeSpace,
    NodeLinear,
    NodeClamp,
    NodeSum,
    NodeMean,
    NodeWave,
    NodePulser,
]

Node.prepclasses(nodeclasses)

def compileall(trees):
    roots = []
    defmap = {}
    for term in trees:
        root = compile(term, implicit=Implicit.SPACE, defmap=defmap)
        roots.append(( root, term.name ))
        if term.name is not None:
            if term.name in defmap:
                raise Exception('duplicate def: %s' % (term.name,))
            defmap[term.name.lower()] = root

    startnod = None
    for (nod, name) in roots:
        if name is None:
            if startnod is not None:
                raise Exception('more than one start')
            startnod = nod
    return Program(startnod, defmap)

def compile(term, implicit, defmap):
    if term.tok.typ == TokType.NUM:
        if term.args:
            raise Exception('number cannot have args')
        return NodeConstant(implicit, asnum=term.tok.val)
    if term.tok.typ != TokType.SYMBOL:
        raise Exception('non-symbol')
    key = term.tok.val.lower()
    if key in defmap:
        if term.args:
            raise Exception('variable name cannot have args: %s' % (key,))
        return defmap[key]
    cla = Node.allclassmap.get(key)
    if not cla:
        raise Exception('unknown term: %s' % (key,))
    nod = cla(implicit)
    nod.parseargs(term.args, defmap=defmap)
    return nod

# Late imports
from program import Program, AxisDep, axisdepname
