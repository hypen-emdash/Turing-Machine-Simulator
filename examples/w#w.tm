read - degenerate case
    ([read], \"\") -> (v, \"\", 0)
    ([read], \"#\") -> ([end], \"#\", +)

read - good cases
    ([read], \"0\")->([search 0 left], \"x\", +)
    ([read], \"1\")->([search 1 left], \"x\", +)

search 0 left
    ([search 0 left], \"0\") -> ([search 0 left], \"0\", +)
    ([search 0 left], \"1\") -> ([search 0 left], \"1\", +)
    ([search 0 left], \"#\") -> ([search 0 right], \"#\", +)
    ([search 0 left], \"\") -> (v, \"\", 0)

search 1 left
    ([search 1 left], \"0\") -> ([search 1 left], \"0\", +)
    ([search 1 left], \"1\") -> ([search 1 left], \"1\", +)
    ([search 1 left], \"#\") -> ([search 1 right], \"#\", +)
    ([search 1 left], \"\") -> (v, \"\", 0)

search 0 right
    ([search 0 right], \"0\") -> ([back up right], \"x\", -)
    ([search 0 right], \"x\") -> ([search 0 right], \"x\", +)
    ([search 0 right], \"1\") -> (v, \"1\", 0)
    ([search 0 right], \"#\") -> (v, \"#\", 0)
    ([search 0 right], \"\") -> (v, \"\", 0)

search 1 right
    ([search 1 right], \"1\") -> ([back up right], \"x\", -)
    ([search 1 right], \"x\") -> ([search 1 right], \"x\", +)
    ([search 1 right], \"0\") -> (v, \"0\", 0)
    ([search 1 right], \"#\") -> (v, \"#\", 0)
    ([search 1 right], \"\") -> (v, \"\", 0)

back up
    ([back up right], \"x\") -> ([back up right], \"x\", -)
    ([back up right], \"#\") -> ([back up left], \"#\", -)
    ([back up left], \"0\") -> ([back up left], \"0\", -)
    ([back up left], \"1\") -> ([back up left], \"1\", -)
    ([back up left], \"x\") -> ([read], \"x\", +)

check at the end
    ([end], \"x\") -> ([end], \"x\", +)
    ([end], \"\") -> (^, \"\", 0)
    ([end], \"0\") -> (v, \"0\", 0)
    ([end], \"1\") -> (v, \"1\", 0)
