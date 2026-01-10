class Program:
    def __init__(self, start, defs):
        self.start = start
        self.defs = defs

        self.nodes = []
        self.nodeidset = set()

    def post(self):
        self.postiter(self.start)
        #print(self.nodes)

    def postiter(self, nod):
        if nod.id in self.nodeidset:
            return
        self.nodes.insert(0, nod)
        self.nodeidset.add(nod.id)
        ### recurse on nod.args

    def write(self):
        print('var clock = 0   // seconds')
        print('%s_pixels = array(pixelCount)' % (self.start.id,))
        print()
        
        print('export function beforeRender(delta) {')
        # delta is ms since last call
        ### we'll want an accuracy hack here
        print('  clock += (delta / 1000)')
        self.start.generatedata()
        print('}')
        print()

        print('export function render(index) {')
        print('  var val = %s_pixels[index]' % (self.start.id,))
        print('  rgb(val*val, 0, 0.1)')
        print('}')
        print()
        
