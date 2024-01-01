gradient
  stop=$00F
  stop=$DDF
  sum
    mul
      wavecycle: sawdecay
        pos=changing: 0, 0.21
        min=0.2, max=0.4
        period=0.03
      pulser
        interval=randnorm: 2, 0.3
        timeshape=flat
        spaceshape=trapezoid
        pos=quote: changing: -0.3, 0.2
        width=randnorm: 0.3, 0.05
    mul
      wavecycle: sawdecay
        pos=changing: 0, 0.11
        min=0.4, max=0.5
        period=0.06
      pulser
        interval=randnorm: 8, 2
        timeshape=flat
        spaceshape=trapezoid
        pos=quote: changing: -0.6, 0.1
        width=randnorm: 0.65, 0.1
    
