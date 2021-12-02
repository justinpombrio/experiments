-- |A generator of values of type a. Each value has a _size_, and you can ask the generator for all
-- |values of a particular size, or for a uniformly random value of a particular size, or related
-- |questions.

-- In `Gen g cs`, `g` is a function from size to all values of that size,
-- and `cs` is a list, whose n'th element is the number of values of size n.
-- INVARIANT: forall (Gen g cs), forall n, length g n = cs !! n
data Gen a = Gen (Int -> [a]) [Integer]

{----------------}
{- Constructors -}
{----------------}

-- |A generator that makes values from the set; the values will all be considered to have size 0.
-- |The set must be finite.
gSet :: [a] -> Gen a
gSet vals =
  Gen (\n -> if n == 0 then vals else [])
      (toInteger (length vals) : repeat 0)

-- |A generator that makes only a single value, considered to be size 0.
gVal :: a -> Gen a
gVal v = gSet [v]

-- |Take two generators, and produce a generator over all possible pairs of their values.
gPair :: Gen a -> Gen b -> Gen (a, b)
gPair (Gen g1 c1) (Gen g2 c2) =
  Gen (\n -> if n < 1
             then []
             else concatMap
                    (\(i, j) -> [(v1, v2) | v1 <- g1 i, v2 <- g2 j])
                    [(i, n - 1 - i) | i <- [0 .. n - 1]])
      (map (\n -> if n < 1
                  then 0
                  else sum [c1 !! i * c2 !! (n - 1 - i) | i <- [0 .. n - 1]])
           [0..])

-- |Take two generators, and produce a generator over the union of their values.
gOr :: Gen a -> Gen a -> Gen a
gOr (Gen g1 cs1) (Gen g2 cs2) =
  Gen (\n -> g1 n ++ g2 n)
      (zipWith (+) cs1 cs2)

infixl 2 <||>
(<||>) = gOr

-- |Increase the size of the values of a generator.
gSize :: Int -> Gen a -> Gen a
gSize m (Gen g cs) | m >= 0 =
  Gen (\n -> g (n - m))
      (replicate m 0 ++ cs)

-- |Modify the values produces by a generator.
gMap :: (a -> b) -> Gen a -> Gen b
gMap f (Gen g cs) = Gen (\n -> map f (g n)) cs

-- |Wrap this around recursive generators, so that they terminate.
gDelay :: Gen a -> Gen a
gDelay gen = Gen (\n -> let Gen g _ = gen in g n) (let Gen _ c = gen in c)

-- |A generator over natural numbers, where number n has size n.
gNat :: Gen Int
gNat = Gen (\n -> [n]) (repeat 1)

{-----------}
{- Queries -}
{-----------}

-- A (possibly infinite) list of all generated values, in order of size.
qAll :: Gen a -> [a]
qAll (Gen g _) = concatMap g [0..]


-- List all values of the given size
qList :: Int -> Gen a -> [a]
qList n (Gen g _) = g n

-- Compute the number of values of the given size
qCount :: Int -> Gen a -> Integer
qCount n (Gen _ cs) = cs !! n

{--------------}
{- TEST CASES -}
{--------------}

data BinaryTree =
    L
  | B BinaryTree BinaryTree
  deriving Show

gBinaryTree :: Gen BinaryTree
gBinaryTree = gDelay $
       gVal L
  <||> gMap (uncurry B) (gPair gBinaryTree gBinaryTree)

gNatTriple :: Gen (Int, Int, Int)
gNatTriple =
  gMap (\(a, (b, c)) -> (a, b, c))
       (gPair gNat (gPair gNat gNat))

main = do
  putStrLn "Number of triples of size 1000:"
  putStrLn $ show $ qCount 1000 gNatTriple
  putStrLn ""
  putStrLn "First triple of size 10000:"
  putStrLn $ show $ (!! 0) $ qList 10000 gNatTriple
  putStrLn ""
  putStrLn "Number of binary trees of size 30:"
  putStrLn $ show $ qCount 30 gBinaryTree
  putStrLn ""
  putStrLn "First binary tree of size 30:"
  putStrLn $ show $ (!! 0) $ qList 30 gBinaryTree
