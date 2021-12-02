module PEG.Stream where

import Prelude hiding (pred)
import Control.Monad (liftM)

data Pos = Pos String Int Int -- filename, line, column

data Stream tok = Stream [[tok]] Pos


stream :: String -> [[tok]] -> Stream tok
stream filename lines = Stream lines (Pos filename 1 1)

takeChar :: Stream tok -> Maybe (tok, Stream tok)
takeChar (Stream [] _) = Nothing
takeChar (Stream ([]:ls) p) = takeChar (Stream ls (advLine p))
takeChar (Stream ((c:l):ls) p) = Just (c, Stream (l:ls) (advCol p))

pred :: (tok -> Bool) -> Stream tok -> Maybe (tok, Stream tok)
pred f s = do
  (c, s) <- takeChar s
  if f c
    then Just (c, s)
    else Nothing

match :: Eq tok => tok -> Stream tok -> Maybe (Stream tok)
match t s = liftM snd (pred (== t) s)

matches :: Eq tok => [tok] -> Stream tok -> Maybe (Stream tok)
matches [] s = Just s
matches (t:ts) s = match t s >>= matches ts

atEof :: Stream tok -> Bool
atEof (Stream ([]:xss) p) = atEof (Stream xss p)
atEof (Stream [] _) = True
atEof _ = False

streamPos (Stream _ p) = p

advLine (Pos f l c) = Pos f (l + 1) c
advCol (Pos f l c) = Pos f l (c + 1)

posKey (Pos f l c) = (f, l, c)
streamKey (Stream _ x) = x

instance Eq Pos where
  x == y = posKey x == posKey y

instance Ord Pos where
  compare x y = compare (posKey x) (posKey y)

instance Eq (Stream tok) where
  x == y = streamKey x == streamKey y

instance Ord (Stream tok) where
  compare x y = compare (streamKey x) (streamKey y)

instance Show Pos where
  showsPrec _ (Pos fileName line column) =
    showString "In file " . showString fileName .
    showString " at line " . shows line .
    showString ", column " . shows column
