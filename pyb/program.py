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

class Stanza:
    def __init__(self, nod):
        self.nod = nod
        self.depend = nod.depend
        self.storedvals = []
        self.bottomline = None
    
    def store_val(self, nod, key, expr):
        varname = '%s_val_%s' % (nod.id, key,)
        self.storedvals.append( (varname, expr) )
        return varname

    def generatebuffer(self):
        self.bottomline = self.nod.generateexpr(ctx=self)

class Program:
    def __init__(self, start, defs):
        self.start = start
        self.defs = defs

        self.nodes = []
        self.nodeidset = set()

        self.stanzas = []

    def post(self):
        self.postiter(self.start)
        assert(self.start is self.nodes[-1])
        self.start.buffered = True

        for nod in self.nodes:
            if nod.buffered:
                stanza = Stanza(nod)
                self.stanzas.append(stanza)
                stanza.generatebuffer()

    def postiter(self, nod):
        if nod.id in self.nodeidset:
            return
        self.nodes.insert(0, nod)
        self.nodeidset.add(nod.id)

        subdeps = AxisDep.NONE

        for argf in nod.argformat:
            argls = nod.getargls(argf.name, argf.multiple)
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

        for argf in nod.argformat:
            argls = nod.getargls(argf.name, argf.multiple)
            for arg in argls:
                if isinstance(arg, Node):
                    if arg.depend != nod.depend and not isinstance(arg, NodeConstant):
                        arg.buffered = True
        
    def dump(self):
        for name in self.defs:
            self.defs[name].dump(name=name)
        self.start.dump()

    def write(self):
        print('var clock = 0   // seconds')
        for stanza in self.stanzas:
            ### or scalar
            print('%s_pixels = array(pixelCount)' % (stanza.nod.id,))
        print()

        for stanza in self.stanzas:
            if not (stanza.depend & AxisDep.TIME):
                ### or no loop
                print('for (var ix=0; ix<pixelCount; ix++) {')
                for varname, expr in stanza.storedvals:
                    print('  var %s = %s' % (varname, expr,))
                print('  %s_pixels[ix] = (%s)' % (stanza.nod.id, stanza.bottomline,))
                print('}')
        print()
        
        print('export function beforeRender(delta) {')
        # delta is ms since last call
        ### we'll want an accuracy hack here
        print('  clock += (delta / 1000)')
        
        for stanza in self.stanzas:
            if stanza.depend & AxisDep.TIME:
                ### or no loop
                print('  for (var ix=0; ix<pixelCount; ix++) {')
                for varname, expr in stanza.storedvals:
                    print('    var %s = %s' % (varname, expr,))
                print('    %s_pixels[ix] = (%s)' % (stanza.nod.id, stanza.bottomline,))
                print('  }')
        print('}')
        print()

        print('export function render(index) {')
        print('  var val = %s_pixels[index]' % (self.start.id,))
        print('  rgb(val*val, 0, 0.1)')
        print('}')
        print()
        


# Late imports
from compile import Node, NodeConstant, Ctx


