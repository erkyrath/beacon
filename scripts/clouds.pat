gradient
  stop=$00F
  stop=$FFF
  sum
    mul
      sum
        0.25
        noise: grain=16, octaves=3
          offset=changing: 0, 0.21
          max=0.25
      pulser
        interval=randnorm: 2, 0.3
        timeshape=flat
        spaceshape=trapezoid
        pos=quote: changing: -0.3, 0.2
        width=randnorm: 0.3, 0.05
    mul
      sum
        0.25
        noise: grain=16, octaves=3
          offset=changing: 0, 0.11
          max=0.25
      pulser
        interval=randnorm: 8, 2
        timeshape=flat
        spaceshape=trapezoid
        pos=quote: changing: -0.6, 0.1
        width=randnorm: 0.65, 0.1
        
