gradient
  stop=$000
  stop=$500
  stop=$900
  stop=$D20
  stop=$EA0
  stop=$EE0
  stop=$EEF
  decay: halflife=0.15
    clamp
      sum
        pulser
            interval=randnorm: 1.0, 0.2
            timeshape=flat
            spaceshape=trapezoid
            width=0.033
            pos=quote: changing: -0.2, 0.5
        pulser
            interval=randnorm: 0.77, 0.2
            timeshape=flat
            spaceshape=trapezoid
            width=0.033
            pos=quote: changing: -0.2, 0.4
        

