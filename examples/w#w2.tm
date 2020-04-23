Initial State: [read left].
Degenerate case
    ([read left], \"\") -> (v, \"\", \"\", 0)
Main cases
    ([read (left)], \"0\") -> ([read 0 on left (left)], \"X\", +)
    ([read (left)], \"1\") -> ([read 1 on left (left)], \"X\", +)
End case
    ([read (left)], \"#\") -> ([check right clear], \"#\", +)

After reading a symbol on left, check it against the first symbol on the right.
    ([read 0 on left (left)], \"0\") -> ([read 0 on left (left)], \"0\", +)
    ([read 0 on left (left)], \"1\") -> ([read 0 on left (left)], \"1\", +)
    ([read 0 on left (left)], \"#\") -> ([read 0 on left (right)], \"#\", +)
    ([read 0 on left (left)], \"\") -> (v, \"\", 0)

    ([read 1 on left (left)], \"0\") -> ([read 1 on left (left)], \"0\", +)
    ([read 1 on left (left)], \"1\") -> ([read 1 on left (left)], \"1\", +)
    ([read 1 on left (left)], \"#\") -> ([read 1 on left (right)], \"#\", +)
    ([read 1 on left (left)], \"\") -> (c \"\", 0)

    ([read 0 on left (right)], \"X\") -> ([read 0 on left (right)], \"X\", +)
    ([read 0 on left (right)], \"0\") -> ([read (right)], \"X\", +)
    ([read 0 on left (right)], \"1\") -> (v, \"1\", 0)
    ([read 0 on left (right)], \"\") -> (v, \"\", 0)

    ([read 1 on left (right)], \"X\") -> ([read 1 on left (right)], \"X\", +)
    ([read 1 on left (right)], \"0\") -> (v, \"0\", 0)
    ([read 1 on left (right)], \"1\") -> ([read (right)], \"X\", +)
    ([read 1 on left (right)], \"\") -> (v, \"\", 0)

And then we do the same thing, but right-to-left.
Main cases
    ([read (right)], \"0\") -> ([read 0 on right (right)], \"X\", -)
    ([read (right)], \"1\") -> ([read 1 on right (right)], \"X\", -)
End case
    ([read (right)], \"\") -> (^, \"\", 0)

After reading a symbol on right, check it against the *leftest* symbol on the left.
    ([read 0 on right (right)], \"X\") -> ([read 0 on right (right)], \"X\", -)
    ([read 0 on right (right)], \"#\") -> ([read 0 on right (left)], \"#\", -)

    ([read 1 on right (right)], \"X\") -> ([read 1 on right (right)], \"X\", -)
    ([read 1 on right (right)], \"#\") -> ([read 1 on right (left)], \"#\", -)

    ([read 0 on right (left)], \"X\") -> ([affirm 0 on left], \"X\", +)
    ([read 0 on right (left)], \"0\") -> ([read 0 on right (left)], \"0\", -)
    ([read 0 on right (left)], \"1\") -> ([read 0 on right (left)], \"1\", -)
    ([read 0 on right (left)], \"\") -> (v, \"\", 0)
    ([read 0 on right (left)], \"#\") -> (v, \"#\", 0)

    ([affirm 0 on left], \"0\") -> ([read (left)], \"X\", +)
    ([affirm 0 on left], \"1\") -> (v, \"1\", 0)
    ([affirm 0 on left], \"#\") -> (v, \"#\", 0)

    ([read 1 on right (left)], \"X\") -> ([affirm 1 on left], \"X\", +)
    ([read 1 on right (left)], \"1\") -> ([read 1 on right (left)], \"1\", -)
    ([read 1 on right (left)], \"0\") -> ([read 1 on right (left)], \"0\", -)
    ([read 1 on right (left)], \"\") -> (v, \"\", 0)
    ([read 1 on right (left)], \"#\") -> (v, \"#\", 0)

    ([affirm 1 on left], \"1\") -> ([read (left)], \"X\", +)
    ([affirm 1 on left], \"0\") -> (v, \"0\", 0)
    ([affirm 1 on left], \"#\") -> (v, \"#\", 0)

At the very end.
    ([check right clear], \"\") -> (^, \"\", 0)
    ([check right clear], \"X\") -> ([check right clear], \"X\", +)
    ([check right clear], \"#\") -> (v, \"\", 0)
    ([check right clear], \"0\") -> (v, \"\", 0)
    ([check right clear], \"1\") -> (v, \"\", 0)
