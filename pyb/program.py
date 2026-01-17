from enum import IntEnum

class AxisDep(IntEnum):
    NONE  = 0
    TIME  = 1
    SPACE = 2
    SPACETIME = 3

def axisdepname(dep):
    match dep:
        case AxisDep.NONE:
            return 'NONE'
        case AxisDep.TIME:
            return 'TIME'
        case AxisDep.SPACE:
            return 'SPACE'
        case AxisDep.SPACETIME:
            return 'SPACETIME'
        case _:
            return '???%s' % (dep,)

class WriteCtx:
    def __init__(self):
        self.storedvals = []

    def store_val(self, nod, key, expr):
        varname = '%s_val_%s' % (nod.id, key,)
        self.storedvals.append( (varname, expr) )
        return varname
        
class Program:
    def __init__(self, start, defs):
        self.start = start
        self.defs = defs

        self.nodes = []
        self.nodeidset = set()

    def post(self):
        self.postiter(self.start)
        assert(self.start is self.nodes[-1])
        self.start.buffered = True
        #print(self.nodes)

    def postiter(self, nod):
        if nod.id in self.nodeidset:
            return
        self.nodes.insert(0, nod)
        self.nodeidset.add(nod.id)

        subdeps = AxisDep.NONE

        for argf in nod.argformat:
            arg = nod.getarg(argf.name)
            if not argf.multiple:
                argls = [ arg ]
            else:
                argls = arg
            for arg in argls:
                if isinstance(arg, Node):
                    self.postiter(arg)
                    subdeps |= arg.depend

        if nod.usesimplicit:
            if nod.implicit == Ctx.TIME:
                nod.depend = AxisDep.TIME
            if nod.implicit == Ctx.SPACE:
                nod.depend = AxisDep.SPACE
        nod.depend |= subdeps

    def dump(self):
        for name in self.defs:
            self.defs[name].dump(name=name)
        self.start.dump()

    def writebuffer(self, nod, ctx):
        val = nod.generatedata(ctx=ctx)
        print('  for (var ix=0; ix<pixelCount; ix++) {')
        for varname, expr in ctx.storedvals:
            ### nod-descended only? or clear after we dump them?
            ### subject to TIME/SPACE placement!
            print('    var %s = %s' % (varname, expr,))
        print('    %s_pixels[ix] = (%s)' % (nod.id, val,))
        print('  }')

    def write(self):
        ctx = WriteCtx()
        
        print('var clock = 0   // seconds')
        for nod in self.nodes:
            if nod.buffered:
                print('%s_pixels = array(pixelCount)' % (nod.id,))
        print()

        ### if nod.buffered and not time-dependent
        print()
        
        print('export function beforeRender(delta) {')
        # delta is ms since last call
        ### we'll want an accuracy hack here
        print('  clock += (delta / 1000)')
        for nod in self.nodes:
            if nod.buffered:
                ### and time-dependent
                self.writebuffer(nod, ctx=ctx)
        print('}')
        print()

        print('export function render(index) {')
        print('  var val = %s_pixels[index]' % (self.start.id,))
        print('  rgb(val*val, 0, 0.1)')
        print('}')
        print()
        


# Late imports
from compile import Node, Ctx


