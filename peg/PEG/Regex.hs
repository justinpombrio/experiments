{-# LANGUAGE Rank2Types, RecursiveDo #-}
module PEG.Regex (regex) where

import PEG.Parser
import PEG.Sugar
import Control.Monad (liftM, liftM2, sequence)
import Data.Map (Map)
import qualified Data.Map as Map

{-
  A super simple regex library.
  I can't find a simple & sane regex standard, so I'm making my own.
  It's basically POSIX ERE, but without support for '\n' or '{m, n}',
    and with fewer weird edge cases (like /[^]]/).
  If you need advanced features, use a PEG instead.

  Regexes e are defined by,
    e ::= e! | e* | e+ | e? | e|e | (e) | [e] | . | c-c | c
    c ::= << any char other than !*+?|()[]\- >>
        | << a backslash followed by one of the above >>
  with mostly the usual semantics (see notes below).

  Notes:
    '[e]' denotes the *choice* between the exprs it contains,
          rather than their *sequence*.
    '!' means negation! e.g. [^a-z] would be written [a-z]!
    '.' does match newlines
    '^' doesn't make sense here
    '$' is not supported
-}

regex :: String -> forall s. GenParser s Char String
regex str = case parse regexGrammar "[regex]" [str] of
  Nothing -> error ("Invalid regex: " ++ str)
  Just r -> r

dangerousChars = "!*+?|()[]-\\"

newlineChars = "\n\r\f"

singletonSatisfy p = liftM (\x -> [x]) (satisfy p)

disjunction [] = Fail
disjunction (x:xs) = x <|> disjunction xs

predicateOr [] x = False
predicateOr (p:ps) x = p x || predicateOr ps x

regexGrammar :: PEG.Sugar.Grammar (GenParser s Char String)
regexGrammar = grammar $ do
  rec
    start <- rule $ do
      x <- compoundExpr
      eof
      return x
    
    compoundExpr <- rule (choice <|> expr)

    choice <- rule $ do
      x <- expr
      char '|'
      y <- expr
      return (x <|> y)
        
    expr <- rule $ do
      xs <- many compoundTerm
      return (liftM concat (sequence xs))
    
    compoundTerm <- rule $
      bang <|> star <|> plus <|> question <|> term
    
    term <- rule $
      dot <|> group <|> bracket <|> range <|> aChar
    
    bang <- rule $ do
      x <- term
      char '!'
      return (Not x <.> succeed "")

    star <- rule $ do
      x <- term
      char '*'
      return (liftM concat (many x))

    plus <- rule $ do
      x <- term
      char '+'
      return (liftM concat (some x))

    question <- rule $ do
      x <- term
      char '?'
      return (liftM (maybe [] id) (option x))

    dot <- rule $ do
      char '.'
      return (singletonSatisfy (const True))

    group <- rule $ do
      char '('
      x <- compoundExpr
      char ')'
      return x
    
    bracket <- rule $ do
      char '['
      xs <- many compoundTerm
      char ']'
      return (disjunction xs)
        
    range <- rule $ do
      c1 <- anyChar
      char '-'
      c2 <- anyChar
      return (singletonSatisfy (\c -> c1 <= c && c <= c2))

    aChar <- rule $ do
      c <- anyChar
      return (chars [c])

    anyChar <- rule (legalChar <|> escapedChar)
    
    legalChar <- rule (satisfy (\c -> not (elem c dangerousChars)))
    
    escapedChar <- rule (char '\\' <.> satisfy (\c -> elem c dangerousChars))


{-
    escapeSeq <- rule (char '\\' <.>
                       (do c <- satisfy (\c -> Map.member c escapeMap)
                           case Map.lookup c escapeMap of
                             Nothing -> error "Regex.escapeSeq: impossible!"
                             Just c -> char c))
-}
  return start
