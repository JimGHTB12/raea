# One gate's result as a JSON line, with the key numbers read from its log (bench/ci.sh runs this after each gate):
#   jq -nc --arg gate GATE --arg status STATUS --arg secs SECONDS --arg log LOG --rawfile text LOG|/dev/null \
#     -f bench/ci/record.jq          (env: CI_DATE, CI_COMMIT, CI_TIER, CI_LOAD_START, CI_LOAD_END, CI_QUIET, RETRY)
# STATUS is PASS, FAIL, FLAKY or SKIP. The numbers are whatever the gate's PASS/FAIL lines carry (frame time p99s,
# SSIMs, loudness, failing counts, test counts); "lines" keeps those lines verbatim.
# (jq's regexes are Oniguruma's Ruby syntax: ^ and $ match at every line, and . never crosses a newline.)

def num: try tonumber catch null;
def int: tonumber;
# every match's capture groups, as strings (null for a group that didn't take part)
# Python's re.M: a leading ^ and a trailing $ hold at every line
def ml: sub("^\\^"; "(?:\\A|(?<=\\n))") | sub("\\$$"; "(?=\\n|\\z)");
def groups($re): [$text | match($re | ml; "g") | [.captures[].string]];
def first_groups($re): (groups($re) | .[0]) // null;
# Python's round(x, 1): half to even on x's exact value (fma takes x * 10 - the tie without rounding)
def round1: . as $x | ($x * 10 | floor) as $f | fma($x; 10; -($f + 0.5)) as $r
  | (if $r > 0 then $f + 1 elif $r < 0 then $f elif ($f % 2) == 0 then $f else $f + 1 end) / 10;

# Python's splitlines, then rstrip
([$text | splits("\r\n|[\n\r\u000b\u000c\u001c\u001d\u001e\u0085\u2028\u2029]")] | if last == "" then .[:-1] else . end | map(sub("\\s+$"; ""))) as $all
| [$all[] | select(test("\\b(PASS|FAIL|FAILED|REGRESSIONS|SKIP)\\b|^gate: |^frames: |failed \\(see|test result:"))] as $lines
| if ($gate | startswith("unit:")) or ($gate | startswith("build:")) then
    ([groups("test result: \\w+\\. (\\d+) passed")[] | .[0] | int] | add // 0) as $p
    | ([groups("(\\d+) failed;")[] | .[0] | int] | add // 0) as $fl
    | (if $p > 0 or $fl > 0 then {passed: $p, failed: $fl} else {} end)
    | (groups("Finished `release` profile .* in (?:(\\d+)m )?([0-9.]+)s") | last) as $m
    | if $m then .cargo_s = ((($m[0] // "0") | int) * 60 + ($m[1] | int) | round1) else . end
    | {num: ., lines: $lines}
  elif $gate | startswith("games") then
    {num: (reduce groups("^(\\w+): (PASS|FAIL)\\s+frames (\\d+) presented, (\\d+) missed; work p99 ([0-9.]+) ms.*?; input p99 ([0-9.]+) frames.*?; rss .*?\\(max (\\d+)\\)")[] as $g
      ({}; .[$g[0]] = {presented: ($g[2] | int), missed: ($g[3] | int), work_p99_ms: ($g[4] | num),
                       input_p99_frames: ($g[5] | num), rss_max_mb: ($g[6] | int)})), lines: $lines}
  elif $gate == "replay" then
    {num: (reduce groups("^(PASS|FAIL) (\\w+):.*\\(worst ([0-9.]+)")[] as $g ({}; .[$g[1]] = {worst_ssim: ($g[2] | num)})), lines: $lines}
  elif $gate == "audio" then
    {num: ({}
      | (first_groups("pong: \\w+\\s+live (-?[0-9.]+) LUFS, true peak (-?[0-9.]+) dBTP; render (-?[0-9.]+) LUFS") as $m
         | if $m then .pong = {live_lufs: ($m[0] | num), live_true_peak: ($m[1] | num), render_lufs: ($m[2] | num)} else . end)
      | (first_groups("tones: \\w+\\s+live clicks (\\d+) \\(render (\\d+)\\)") as $m
         | if $m then .tones = {live_clicks: ($m[0] | int), render_clicks: ($m[1] | int)} else . end)), lines: $lines}
  elif $gate == "tone" then
    {num: (reduce groups("^(\\w+): (PASS|FAIL)\\s+bounce vs live max diff (\\S+) over ([0-9.]+) s")[] as $g
             ({}; .[$g[0]] = {max_diff: ($g[2] | num), seconds: ($g[3] | num)})
      | (first_groups("ducking: \\w+\\s+the music under the voice (-?[0-9.]+) dB") as $m | if $m then .ducking_db = ($m[0] | num) else . end)),
     lines: $lines}
  elif $gate == "timelines" then
    {num: (reduce ($text | match("^([\\w, ]+?): (PASS|FAIL) .*\\(worst ([0-9.]+)\\)(?:; opened and baked in (\\d+) ms)?" | ml; "g")) as $m ({};
      ([$m.captures[].string]) as $g
      | ({worst_ssim: ($g[2] | num)}
         | if $g[3] then .bake_ms = ($g[3] | int) else . end
         | (first(($m.string | match("\\((\\d+) of (\\d+) frames stepped, (\\d+) ms\\)")) | [.captures[].string]) // null) as $inc
         | if $inc then .incremental = {stepped: ($inc[0] | int), of: ($inc[1] | int), ms: ($inc[2] | int)} else . end) as $e
      | .[$g[0] | gsub(", "; "_") | gsub(" "; "_")] = $e)), lines: $lines}
  elif $gate | startswith("video") then
    {num: (reduce groups("^([\\w.-]+): (PASS|FAIL)\\s+scrub p99 ([0-9.]+) ms.*?missed (\\d+) of (\\d+).*?drift max ([0-9.]+|none).*?rss max (\\d+)")[] as $g
      ({}; .[$g[0]] = {scrub_p99_ms: ($g[2] | num), missed: ($g[3] | int), presented: ($g[4] | int),
                       drift_max_ms: ($g[5] | num), rss_max_mb: ($g[6] | int)})), lines: $lines}
  elif $gate == "substrate" then
    {num: {failing: (first_groups("^gate: (\\d+) failing") | if . then .[0] | int else null end),
           passing: ([$text | match("^PASS " | ml; "g")] | length)}, lines: $lines}
  elif $gate | startswith("parity") then
    ([groups("^  (\\S+ @[0-9.]+s: [0-9.]+ -> [0-9.]+)$")[] | .[0]]) as $regs
    | {num: (first_groups("frames: (\\d+) exact, (\\d+) ok, (\\d+) fail; cases without result: (\\d+)") as $m
             | (if $m then {exact: ($m[0] | int), ok: ($m[1] | int), fail: ($m[2] | int), no_result: ($m[3] | int)} else {} end)
             | .regressions = ($regs | length)),
       lines: ([$lines[] | select(contains("--") | not)] + $regs[:20])}
  else {num: {}, lines: $lines} end
| {date: $ENV.CI_DATE, at: (now | strftime("%Y-%m-%dT%H:%M:%SZ")), commit: $ENV.CI_COMMIT, tier: $ENV.CI_TIER,
   gate: $gate, status: $status, attempt: (if ($ENV.RETRY // "") != "" then 2 else 1 end), seconds: ($secs | tonumber | round1),
   load: {start: ($ENV.CI_LOAD_START | num), end: ($ENV.CI_LOAD_END | num)},
   quiet: ({"true": true, "false": false}[$ENV.CI_QUIET // ""]), numbers: .num, lines: .lines[:40], log: $log}
