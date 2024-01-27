# `cmb`

An interpreter for combinatorial expressions.

## Defining new combinators
```
:SK>I=SK*
:SKI>I
Parsed `I` of size 1 into `SK*` of size 3
:SKI>Ix
Parsed `Ix` of size 2 into `x` of size 1
```

## Getting new combinators for free
The arguments `-B`, `-W`, `-C`, and `-I`
add built-in `B`, `W`, `C`, and `I` combinators.

`-S` and `-K` remove the `S` and `K` combinators.

### Combinator definitions
```
Sxyz=xz(yz)
Kxy=x
Ix=x
Bxyz=x(yz)
Cxyz=xzy
Wxy=xyy
```
