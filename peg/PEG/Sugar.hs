{-# LANGUAGE Rank2Types #-}
module PEG.Sugar where

import Control.Monad (liftM)
import PEG.Parser
import PEG.Stream (streamPos)

type GenGrammar t a = PEG.Parser.Grammar t a
type GenParser s t a = PEG.Parser.Parser s t a

type Grammar a = GenGrammar Char a
type Parser s a = GenParser s Char a

grammar = PEG.Parser.Grammar


many e = some e <|> return []

some e = do
  x <- e
  xs <- many e
  return (x : xs)

option e = liftM Just e <|> return Nothing

sep d e =
  do {
    x <- e;
    d;
    xs <- sep d e;
    return (x:xs)
  }
  <|> do {
    x <- e;
    return [x]
  }
  

ignore = neg . neg

infixl 4 <|>
(<|>) = Choice

infixl 5 <.>
(<.>) :: GenParser s t a -> GenParser s t b -> GenParser s t b
(<.>) = (>>)

neg = Not
eof = Eof
satisfy = Pred
char t = Match t >> return t
chars ts = Matches ts >> return ts
getPos = liftM streamPos GetStream
succeed = Succeed
failure = Fail