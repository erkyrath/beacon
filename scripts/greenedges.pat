gradient
  stop=$000
  stop=$000
  stop=$040
  stop=$0F0
  stop=$040
  stop=$000
  stop=$000
  mul
      1.5
      clamp: min=0.0, max=0.6666
        pulser
          interval=0.4
          timeshape=sine
          spaceshape=triangle
          pos=randflat: 0, 1
          width=0.2
          duration=4
        