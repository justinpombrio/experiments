{-# LANGUAGE RecursiveDo, Rank2Types #-}
module Main where

import Prelude hiding (fail)
import PEG
import Data.Char (isDigit)
import Control.Monad hiding (fail)
import Data.List
import qualified Data.Map as Map
import Data.STRef
import Data.Char
import Control.Exception (try, ErrorCall)


-- Each test is a pair (list of inputs, expected result for those inputs)
runTests :: (Eq t, Show t, Eq a, Show a) =>
            String -> GenGrammar t a -> [([[t]], Maybe a)] -> IO ()
runTests name grammar tests =
  let flattenGroup (inputs, output) = map (\i -> (i, output)) inputs
      testPairs = concat $ map flattenGroup tests in
  mapM_ (runTest name grammar) testPairs

runTest :: (Eq t, Show t, Eq a, Show a) =>
           String -> GenGrammar t a -> ([t], Maybe a) -> IO ()
runTest testName grammar (input, expectedResult) = do
  let result = parse grammar testName [input]
  if result == expectedResult
    then return ()
    else do
      putStrLn $ "Test " ++ testName ++ " failed."
        ++ "  Input: " ++ (show input)
        ++ "  Expected: " ++ (show expectedResult)
        ++ "  Actual: " ++ (show result)

-- Each test is a pair (input, expected output)
runRegexTest :: String -> [(String, Maybe String)] -> IO ()
runRegexTest re tests =
  runTests ("Regex (" ++ re ++ ")")
           (grammar $ rule $ regex re)
           (map (\(x, y) -> ([x], y)) tests)

checkRegexFailure :: String -> IO ()
checkRegexFailure re = do
  failed <- try (regex re `seq` return ())
  case failed of
    Left e -> return ((e :: ErrorCall) `seq` ())
    Right _ -> putStrLn $ "Regex (" ++ re ++ ") should be invalid!"

checkRegexFailures :: [String] -> IO ()
checkRegexFailures = mapM_ checkRegexFailure

-- The first list is expected passing inputs; second expected failing inputs
runSimpleRegexTest :: String -> [String] -> [String] -> IO ()
runSimpleRegexTest re passing failing =
  runRegexTest re (map (\x -> (x, Just x)) passing ++
                   map (\x -> (x, Nothing)) failing)

pass x = (x, Just x)
fail x = (x, Nothing)
main = do
  runRegexTest "x?y"
    [pass "y",
     pass "xy",
     fail "",
     fail "x",
     ("yx", Just "y"),
     ("xyx", Just "xy"),
     ("xyy", Just "xy")]
  runRegexTest "xy*z+"
    [fail "", fail "x", fail "y", fail "z", fail "xy", fail "yz", fail "xxz",
     pass "xz",
     ("xzx", Just "xz"),
     pass "xyyyz",
     pass "xyzzz",
     pass "xzz",
     ("xyyyyzzzzyyyyzzzz", Just "xyyyyzzzz")]
  runRegexTest "(xy)|z"
    [fail "", fail "x", fail "y", fail "yx", fail "xz", fail "yz", fail "xx",
     ("xyz", Just "xy"),
     ("zz", Just "z"),
     pass "xy", pass "z"]
  runRegexTest "[0-9]"
    [fail "x", fail "-3",
     ("09", Just "0"), ("00", Just "0"), ("45", Just "4"),
     pass "0", pass "1", pass "3", pass "9"]
  runRegexTest "x!"
    [fail "x", pass "", ("y", Just "")]
  runRegexTest "."
    [fail "",
     ("xy", Just "x"),
     ("xx", Just "x"),
     pass "x", pass "y"]
  runRegexTest "b-e"
    [fail "a", fail "f", fail "z", fail "0",
     fail "", fail "ab",
     ("bb", Just "b"),
     ("bc", Just "b"),
     pass "b", pass "c", pass "d", pass "e"]
  checkRegexFailures ["!", "*", "+", "?", "\\",
                      "x-", "-x", "-",
                      "(", ")", "[", "]", "(()", "[]]",
                      "\\a", "\\b", "\\z", "\\0"]
  runRegexTest "\n" [pass "\n", fail "", fail "n", fail "x", fail "\\"]
  runRegexTest "\\!" [pass "!", fail "\\", fail "\\!", fail "x", fail ""]
  runRegexTest "[\\+\\-]?(0|[1-9][0-9]*)"
    [fail "",
     pass "0",
     pass "1",
     pass "-34",
     ("-34x", Just "-34"),
     ("-00", Just "-0"),
     pass "348083",
     pass "+4838",
     fail "++343"]
  
  runTests "Simple" loneA
    [(["a"], Just ()),
     (["", "b", "aa"], Nothing)]
  
  -- See the Catalan numbers?
  runTests "Matching Parens" matchingParens
    [(["()"], Just 1),
     (["(())"], Just 2),
     (["((()))", "(()())"], Just 3),
     (["(((())))", "((()()))", "((())())", "(()(()))", "(()()())"], Just 4),
     (["(", ")",
       "((", ")(", "))",
       "(()", "())", "()()", "(()())()", "(((())())"], Nothing)]

  runTests "Non-context-free" nonContextFree
    [([[1, 2, 3], [1, 1, 2, 2, 3, 3], [1, 1, 1, 2, 2, 2, 3, 3, 3]],
      Just ()),
     ([[1, 1], [1, 2], [1, 3], [2, 1], [2, 2], [2, 3], [3, 1], [3, 2], [3, 3],
       [1, 1, 2, 3], [1, 2, 2, 3], [1, 2, 3, 3],
       [1, 1, 2, 2, 3], [1, 2, 2, 3, 3], [1, 1, 2, 3, 3], [1, 2, 1, 2, 3]],
      Nothing)]
  
  runTests "ML Comments" comments
    [(["(**)", "(*(**)*)", "(*(*(**)*)*)", "(*(**)(**)*)", "(*(**)(*(**)*)*)",
       "(*x*)", "(*comment*)", "(*X(*Y*)Z*)",
       "(*X (* Y*) Z *)", "(*(*X*)X(*(**)*)X*)",
       "(*(x*)", "(***)", "(*)*)", "(*(*)*)((**)**)",
       "(**(*)*)**(*()*(*)*))*)*)", "(*)(***)()(**(**))*)*)"],
      Just ()),
    (["*", "()", "(*", "(*(**))", "(**)*)", "(*(**)(**)",
      "x(**)", "(**)x", "((**)", "(**)*",
      "(*(**)(**)*)*)", "(*(**)*))", "(*(*((*(**))*)*)"],
     Nothing)]
  
  runTests "Memoization" memoTest
   [([concat ["((((((((((((((((((((((((((((((",
              "((((((((((((((((((((((((((((((",
              " o ",
              ")))]))]]]])))]]]))])))])))])]]",
              "))])]]]])]))))]])))]]]]]]]])))"]], Just ())]
  
  runTests "Extended" extended
    [(["3", "3+4"], Just ()),
     (["", "+", "+3", "3+4+5"], Nothing)]

  putStrLn "ok"


loneA = grammar $ do
  rec
    s <- rule $ r <.> eof
    r <- rule $ chars "a"
  
  return s

-- I think this takes exponential time without memoization.
memoTest = grammar $ do
  rec
    s <- rule $ r <.> eof
    r <- rule $ chars " o "
      <|> chars "(" <.> r <.> chars ")"
      <|> chars "(" <.> r <.> chars "]"
  
  return s

-- Matches {1^n 2^n 3^n}; taken from Wikipedia
nonContextFree = grammar $ do
  rec
    abc <- rule $ ignore (ab <.> chars [3]) <.> as <.> bc <.> eof
    as <- rule $ many (chars [1])
    ab <- rule $ chars [1] <.> ab <.> chars [2] <|> chars [1, 2]
    bc <- rule $ chars [2] <.> bc <.> chars [3] <|> chars [2, 3]

  return abc

-- Ensure that parens match, and count the maximum depth.
matchingParens = grammar $ do
  rec
    parens <- rule $ do
       chars "("
       n <- expr
       chars ")"
       return (n + 1)
  
    expr <- rule $ exprBody <|> return 0
    
    exprBody <- rule $ do
      x <- parens
      y <- expr
      return (x + y)
  
    start <- rule $ do
      n <- parens
      eof
      return n
  
  return start

comments = grammar $ do
  rec
    start <- rule $ comment <.> eof
    comment <- rule $ chars "(*" <.> body <.> chars "*)"
    body <- rule $ comment <.> body <|> char <.> body <|> return ()
    text <- rule $ char <.> text <|> return ()
    char <- rule $ neg (chars "(*") <.> neg (chars "*)") <.> satisfy (\_ -> True)
  return start

extensible = do
  rec
    start <- rule ((expr <|> num) <.> eof)
    expr <- extensibleRule failure
    num <- rule (regex "0-9+")
  return (start, expr, num)

extended = grammar $ do
  (start, expr, num) <- extensible
  sum <- rule (num <.> char '+' <.> num)
  extend expr sum
  return start
