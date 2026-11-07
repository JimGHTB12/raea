# Number formatting for lightspeed's harnesses (bench/lightspeed/*.sh, `jq -L bench/lightspeed 'include "lib"; ...'`), so
# their rows read exactly as the Python harnesses' did (the history in results.jsonl stays comparable):
#   pyround($n)  Python's round(x, n): half to even on x's exact value (fma takes x * 10^n - the tie without rounding),
#                n = 0 for round(x)
#   pyf          a float as Python's str() writes it (12.0, not 12)
#   pystr        a number read from JSON as Python's str() writes it: a float if its text has a point or an exponent
#                (jq 1.7 keeps a number's text until it's computed with), an int otherwise; null is None
#   fixed($n)    Python's f"{x:.nf}"
#   pct($p)      the harnesses' percentile: the sorted list's element round((n - 1) * p)
#   median       statistics.median
#   pylist       a list of floats as Python's str() writes it ([1.0, 2.5])

def pyround($n): . as $x | pow(10; $n) as $m | ($x * $m | floor) as $f | fma($x; $m; -($f + 0.5)) as $r
  | (if $r > 0 then $f + 1 elif $r < 0 then $f elif ($f % 2) == 0 then $f else $f + 1 end) / $m;
def pyf: if . == null then "None" else (. + 0 | tostring) as $t | if ($t | test("^-?[0-9]+$")) then $t + ".0" else $t end end;
def pystr: if . == null then "None" elif type == "number" then (if (tostring | test("[.eE]")) then pyf else tostring end)
  elif type == "boolean" then (if . then "True" else "False" end) else tostring end;
def fixed($n): pyround($n) | tostring | split(".") as $p
  | if $n == 0 then $p[0] else $p[0] + "." + ((($p[1] // "") + ([range($n)] | map("0") | join("")))[:$n]) end;
def pct($p): sort as $s | if ($s | length) == 0 then null else $s[(($s | length) - 1) * $p | pyround(0)] end;
def median: sort as $s | ($s | length) as $n | if $n % 2 == 1 then $s[($n - 1) / 2] else ($s[$n / 2 - 1] + $s[$n / 2]) / 2 end;
def pylist: "[" + (map(pyf) | join(", ")) + "]";
