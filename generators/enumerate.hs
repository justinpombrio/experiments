import Test.Feat
import Test.Feat.Finite 
import Test.Feat.Enumerate (parts, fromParts, cartesian, Finite)

data Nat = Nat Int
data Pair = Pair Int Int
data Triple = Triple Int Int Int
  deriving Show

nats :: Enumerate Int
nats = fromParts $ map (\n -> Finite { fCard = 1, fIndex = \i->n }) [0..]

pairs :: Enumerate (Int, Int)
pairs = cartesian nats nats

triples :: Enumerate (Int, (Int, Int))
triples = cartesian nats pairs

main = do
  putStrLn "Number of triples of size 10000 (Enumerate):"
  putStrLn $ show $ fCard $ parts triples !! 10000

instance Enumerable Pair where
  enumerate = datatype [ c2 Pair ]

instance Enumerable Triple where
  enumerate = datatype [ c2 (\x (Pair y z) -> Triple x y z) ]
