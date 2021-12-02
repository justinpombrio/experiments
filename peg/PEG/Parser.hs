{-# LANGUAGE Rank2Types, GADTs #-}
module PEG.Parser where

import Prelude hiding (pred)
import qualified Debug.Trace as Trace
import Control.Monad (liftM)
import Control.Monad.ST
import Control.Monad.State
import Control.Applicative
import qualified Data.Map as Map
import Data.Map (Map)
import Data.STRef
import PEG.Stream


type Memo t a = Map (Stream t) (Result t a)

newtype Grammar t a = Grammar (forall s. ST s (Parser s t a))

type Rule s t a = STRef s (Parser s t a)

data Parser s t a where
  Succeed :: a -> Parser s t a
  Fail :: Parser s t a
  Eof :: Parser s t ()
  Match :: t -> Parser s t ()
  Matches :: [t] -> Parser s t ()
  Pred :: (t -> Bool) -> Parser s t t
  Seq :: Parser s t a -> (a -> Parser s t b) -> Parser s t b
  Choice :: Parser s t a -> Parser s t a -> Parser s t a
  Not :: Parser s t a -> Parser s t ()
  Memo :: Parser s t a -> STRef s (Memo t a) -> Parser s t a
  GetStream :: Parser s t (Stream t)
  SetStream :: Stream t -> Parser s t ()
  Rule :: Rule s t a -> Parser s t a

data Result t a = Failure
                | Success (Stream t) a

rule :: Parser s t a -> ST s (Parser s t a)
rule rule = do
    memo <- newSTRef Map.empty
    return (Memo rule memo)

extensibleRule :: Parser s t a -> ST s (Parser s t a)
extensibleRule e = do
  ref <- newSTRef e
  return (Rule ref)
      
extendRule :: Rule s t a -> Parser s t a -> ST s ()
extendRule ref e = do
  e' <- readSTRef ref
  writeSTRef ref (Choice e' e)

extend :: Parser s t a -> Parser s t a -> ST s ()
extend (Rule ref) e = extendRule ref e
extend _ _ = error "PEG.extend: Can only extend a parse rule."

instance Functor (Parser s t) where
  fmap f a = a `Seq` (Succeed . f)

instance Applicative (Parser s t) where
  pure = Succeed
  mf <*> ma = mf `Seq` (\f -> ma `Seq` (\a -> Succeed (f a)))

instance Monad (Parser s t) where
  --return = Succeed
  (>>=) = Seq

instance Alternative (Parser s t) where
  empty = Fail
  (<|>) = Choice

instance MonadPlus (Parser s t)


parse :: Eq t => Grammar t a -> String -> [[t]] -> Maybe a
parse (Grammar grammar) filename lines =
  let result = runST $ do
        parser <- grammar
        run parser (stream filename lines) in
  case result of
    Failure -> Nothing
    Success _ x -> Just x


run :: Eq t => Parser s t a -> Stream t -> ST s (Result t a)

run (Succeed x) s = return (Success s x)

run Fail s = return Failure

run GetStream s = return (Success s s)

run (SetStream s') s = return (Success s' ())

run Eof s =
  if atEof s
  then return (Success s ())
  else return Failure

run (Match tok) s =
  case match tok s of
    Nothing -> return Failure
    Just s' -> return (Success s' ())

run (Matches toks) s =
  case matches toks s of
    Nothing -> return Failure
    Just s' -> return (Success s' ())

run (Pred p) s =
  case pred p s of
    Nothing -> return Failure
    Just (x, s') -> return (Success s' x)

run (Seq e f) s = do
  result <- run e s
  case result of
    Failure -> return Failure
    Success s' x -> run (f x) s'

run (Choice e e') s = do
  result <- run e s
  case result of
    Failure -> run e' s
    Success s' x -> return (Success s' x)

run (Not e) s = do
  result <- run e s
  case result of
    Failure -> return (Success s ())
    Success _ _ -> return Failure

run (Memo e ref) s = do
  memo <- readSTRef ref
  case Map.lookup s memo of
    Nothing -> do
      result <- run e s
      writeSTRef ref (Map.insert s result memo)
      return result
    Just result -> return result

run (Rule ref) s = do
  e <- readSTRef ref
  run e s
