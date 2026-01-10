class Program:
    def __init__(self, start, defs):
        self.start = start
        self.defs = defs

    def write(self):
        print('var clock = 0   # seconds')
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
        
