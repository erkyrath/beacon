gradient
  stop=$44F
  stop=$44F
  stop=$000
  stop=$C60
  stop=$C60
  mul
      2
      clamp: min=0.0, max=0.5
        pulser
          interval=0.4
          timeshape=triangle
          spaceshape=sine
          pos=randflat: 0, 1
          width=0.15
          duration=3
        