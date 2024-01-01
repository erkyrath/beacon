gradient:
  stop=$000
  stop=$F00
  stop=$000
  stop=$F20
  stop=$000
  stop=$F40
  stop=$000
  stop=$F60
  stop=$000
  stop=$000
  stop=$000
  stop=$000
  stop=$000
  stop=$000
  stop=$000
  stop=$000
  stop=$000
  stop=$000
  pulser:
        interval = 2
        pos = randnorm: 0.5, 0.075
        timeshape = sawdecay
        spaceshape = sine
        width = quote: changing: 0.1, 0.3
        duration = 4.0

