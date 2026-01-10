from enum import IntEnum

class AxisDep(IntEnum):
    NONE  = 0
    TIME  = 1
    SPACE = 2
    SPACETIME = 3


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

        for argf in nod.argformat:
            arg = getattr(nod.args, argf.name)
            if not argf.multiple:
                if isinstance(arg, Node):
                    self.postiter(arg)
            else:
                argls = arg
                for arg in argls:
                    if isinstance(arg, Node):
                        self.postiter(arg)

    def writebuffer(self, nod):
        val = nod.generatedata()
        print('  for (ix=0; ix<pixelCount; ix++) {')
        print('    %s[ix] = (%s)' % (nod.id, val,))
        print('  }')

    def write(self):
        print('var clock = 0   // seconds')
        print('%s_pixels = array(pixelCount)' % (self.start.id,))
        print()

        print('export function atStartup {')
        print('}')
        print()
        
        print('export function beforeRender(delta) {')
        # delta is ms since last call
        ### we'll want an accuracy hack here
        print('  clock += (delta / 1000)')
        for nod in self.nodes:
            if nod.buffered:
                self.writebuffer(nod)
        print('}')
        print()

        print('export function render(index) {')
        print('  var val = %s_pixels[index]' % (self.start.id,))
        print('  rgb(val*val, 0, 0.1)')
        print('}')
        print()
        


# Late imports
from compile import Node


