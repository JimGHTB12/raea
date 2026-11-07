# lightspeed's scoreboard (docs/lightspeed.feature, L0): every row's newest value, its target, the gap and the trend.
# bench/scoreboard.sh runs it:
#   jq -nr -L bench/lightspeed --rawfile results R --rawfile ci C --argjson evals E --argjson behind B --arg json 0|1 \
#     -f bench/lightspeed/scoreboard.jq
# R: ~/Library/Logs/raea/lightspeed/results.jsonl; C: ~/Library/Logs/raea/ci/history.jsonl; E: the legacy app's newest eval run (retired with apps/legacy: null)
# ({file, rows}, or null); B: {commit: commits HEAD is past it} for the CI rows' commits. Sources, newest first:
#   results.jsonl   the lightspeed harnesses (bench/lightspeed/*.sh): each row's newest value, marked "under load N" when
#                   the machine was busy, with its newest quiet value beside it (bench/ci.sh's nightly quiet pass)
#   history.jsonl   bench/ci.sh's gates (Q1, Q2, Q4, T3): quiet results only, marked STALE more than 10 commits behind
#                   HEAD; quality.sh's newer quiet run of the same gate on this tree wins
#   evals           the legacy app's newest eval run (Q6; retired with apps/legacy, so null)
#   carried         the value in lightspeed.feature, for a row no harness could measure yet (the phone locked, say),
#                   marked as carried
# Rows are ranked by gap weighted by how much a person feels them (feel 4, throughput 3, resources 2, quality 1); a row
# whose newest quiet value is >10% worse than the one before it is flagged REGRESSED.
include "lib";

# a JSON line a line; a line that isn't JSON is skipped
def lines: [split("\n")[] | fromjson? | select(type == "object")];
# the first number in a value's text, as a float
def num: pystr | [match("-?\\d+(?:\\.\\d+)?") | .string] | if length == 0 then null else .[0] | tonumber end;
# Python's str() of a value as the scoreboard shows it
def show: if type == "string" then . else pystr end;
def ljust($n): . + ([range([$n - length, 0] | max)] | map(" ") | join(""));
def rjust($n): ([range([$n - length, 0] | max)] | map(" ") | join("")) + .;

# row: [title, target text, target number (null: none), lower is better]; the number the gap is judged on is value($row)
def ROWS: [
  ["F1", "input to present p99 (offscreen; window needs consent)", "<= 1 refresh + 1 frame (25 ms at 60 Hz)", 25.0, true],
  ["F2", "frame work p99 / p99.9, games 120 fps", "p99.9 <= 4 ms", 4.0, true],
  ["F3", "missed frames, pong + platformer", "0", 0.0, true],
  ["F4", "scrub p99, 1080p / 4K", "<= 16.7 / 33 ms", 1.0, true],
  ["F5.game", "spawn to first frame: game", "< 100 ms", 100.0, true],
  ["F5.timeline", "spawn to first frame: timeline (bake inside)", "< 100 ms", 100.0, true],
  ["F5.bake", "timeline bake (titles)", "< 100 ms", 100.0, true],
  ["F6", "first-use hitches over 2 ms", "none", 0.0, true],
  ["T1", "ts1 export, warm", "<= 1 s", 1.0, true],
  ["T2", "3D export, 600 frames", "<= 1 s", 1.0, true],
  ["T3", "bake 20 s edit / incremental re-bake", "<= 50 / 10 ms", 1.0, true],
  ["T4", "batch renders per hour (titles)", "encoder-bound on every core", 1.0, true],
  ["R1", "bytes allocated a frame after warmup", "0 every layer", 0.0, true],
  ["R2", "GC collections during play", "0", 0.0, true],
  ["R4", "GPU memory per stage", "1 texture per page (~4-8 MB)", 1.0, true],
  ["R5", "iPhone peak footprint", "<= 200 MB", 200.0, true],
  ["R7", "exported game size", "<= 65 MB", 65.0, true],
  ["R6", "CPU watts pong / timeline", "pong halved", null, true],
  ["Q1", "render = play, worst SSIM", "1.000000 everywhere", 1.0, false],
  ["Q2", "Chrome parity exact / ok / fail", "all exact", 1.0, false],
  ["Q3.vmaf", "export VMAF vs lossless (ts1)", ">= 95 (mean and worst)", 95.0, false],
  ["Q3.colour", "colour patches after encode, worst 8-bit error", "BT.709 exact (0)", 0.0, true],
  ["Q4", "audio tap = bounce, LUFS, peak, clicks", "exact, -14, 0 clicks", 1.0, false],
  ["Q5", "Mac = iPhone, byte for byte", "byte-identical", 0.0, true],
  ["Q6", "agent first-try pass rate", "measured nightly; rising", 1.0, false]
];
def WEIGHT: {"F": 4, "T": 3, "R": 2, "Q": 1};
def CARRIED: {"R7": ["66.77 MB", "sdk/react/bench/games/export.sh app_MB, 9012fdc (2026-10-18)"],
              "R5": ["373 MB", "sdk/react/ios probe, 4279d3f (2026-10-17)"],
              "Q5": ["frame 0 exact; glyph edges <= 14/255", "sdk/react/ios probe, 4279d3f"]};
# a dict's key a Python lambda would index: missing is an error (the gap is then None)
def key($k): if type == "object" and has($k) then .[$k] else error("KeyError") end;

# the number a row's gap is judged on (an error where Python's lambda raised)
def value($row):
  if $row == "F2" then (.value | show | split("/")) as $p | if ($p | length) < 2 then error("IndexError") else $p[1] | num end
  elif $row == "F4" then key("detail") as $d
    | [(($d.ts1 // {}).scrub_p99_ms // 0) / 16.7, (($d.k4 // {}).scrub_p99_ms // 0) / 33] | max
  elif $row == "T3" then key("n") | [.[0] / 50, .[1] / 10] | max
  elif $row == "T4" then key("detail") | key("cores") / ([key("scaling_vs_1"), 0.01] | max)
  elif $row == "R4" then [key("detail")[] | key("graphics_mb")] | max / 8
  elif $row == "Q3.vmaf" then key("detail") | key("min")
  elif ($row | IN("Q1", "Q2", "Q4", "Q6")) then key("n")
  else .value | num end;

def gap($row):
  (ROWS[] | select(.[0] == $row)) as [$_, $title, $target, $tnum, $lower]
  | (try value($row) catch null) as $v
  | if $v == null or $tnum == null then null
    elif $lower then (if $tnum == 0 then (if $v > 0 then 1.0 + $v else 1.0 end) else [$v / $tnum, 1.0] | max end)
    elif $v != 0 then [$tnum / $v, 1.0] | max else 99.0 end;

($results | lines) as $res
# each row's newest value, quiet or under load (the user, 2026-10-18: rows are marked with their load, not held for a
# quiet machine), and its newest quiet one beside it when that's older (the nightly pass confirms them)
| (reduce $res[] as $r ({best: {}, prev: {}, quiet: {}};
    (if .best[$r.row] then .prev[$r.row] = .best[$r.row] else . end)
    | .best[$r.row] = $r
    | if $r.quiet then .quiet[$r.row] = $r else . end)) as $s
| ($s.best | map_values(. + {src: ((.harness // "") | split("/") | last)})) as $rows0
# CI's gates: quiet results only (a gate timed on a busy machine says nothing about the code), the newest a gate
| (reduce ($ci | lines)[] as $r ({}; if $r.quiet != false then .[$r.gate] = $r else . end)) as $latest
| ({}
   | ($latest.timelines) as $tl
   | if $tl and (($tl.numbers // {}) | length) > 0 then
       ($tl.numbers.titles.bake_ms) as $bake | ($tl.numbers.titles_edited.incremental.ms) as $inc
       | if $bake != null and $inc != null then
           .T3 = {value: "\($bake | pystr) / \($inc | pystr)", unit: "ms (titles; CI)", n: [$bake, $inc], date: $tl.date,
                  commit: $tl.commit[:9], quiet: $tl.quiet, src: "ci timelines"} else . end
     else . end
   | ([("replay", "timelines") as $g | $latest[$g] | select(. and ((.numbers // {}) | length) > 0) as $r
       | $r.numbers | to_entries[] | select((.value | type) == "object" and (.value | has("worst_ssim")))
       | [.value.worst_ssim, "\($g):\(.key)", $r]]) as $ss
   | if ($ss | length) > 0 then ($ss | min_by(.[0:2])) as $w
       | .Q1 = {value: "\($w[0] | fixed(6)) (\($w[1]))", unit: "worst SSIM over CI's replay and timelines", n: $w[0],
                date: $w[2].date, commit: $w[2].commit[:9], quiet: $w[2].quiet, src: "ci"} else . end
   | ([$latest | to_entries[] | select(.key | startswith("parity:")) | .value]) as $par
   | if ($par | length) > 0 then
       ([$par[] | .numbers.exact // 0] | add) as $e | ([$par[] | .numbers.ok // 0] | add) as $o | ([$par[] | .numbers.fail // 0] | add) as $f
       | .Q2 = {value: "\($e) / \($o) / \($f)", unit: "exact / ok / fail (\([$par[].gate] | sort | join(", ")))",
                n: ($e / ([$e + $o + $f, 1] | max)), date: ([$par[].date] | max), commit: $par[-1].commit[:9],
                quiet: (all($par[]; .quiet)), src: "ci"} else . end
   | ($latest.audio) as $a
   | if $a then ($a.numbers // {}) as $n
       | .Q4 = {value: "\($a.status) (pong live \($n.pong.live_lufs | pystr) / render \($n.pong.render_lufs | pystr) LUFS, clicks \($n.tones.live_clicks | pystr))",
                unit: "CI audio gate", n: (if $a.status == "PASS" then 1.0 else 0.5 end), date: $a.date, commit: $a.commit[:9],
                quiet: $a.quiet, src: "ci"} else . end) as $from_ci
| (if $evals == null then {} else
     ([$evals.rows[] | select(type == "object" and has("pass") and (.infra | not))]) as $rs
     | if ($rs | length) == 0 then {} else ([$rs[] | select(.pass)] | length) as $p | ([$rs[] | .cost_usd // 0] | add) as $cost
       | {Q6: {value: "\($p)/\($rs | length) pass", unit: "\($evals.file), $\($cost | fixed(2))", n: ($p / ($rs | length)),
               date: "", commit: "", quiet: true, src: "legacy evals"}} end end) as $from_evals
| (reduce (($from_ci + $from_evals) | to_entries[]) as $e ($rows0;
    .[$e.key] as $mine
    | if $mine and $mine.quiet and (($mine.date // "") >= (($e.value.date // "")[:19])) then .  # quality.sh measured it on this tree since
      else ($behind[$e.value.commit // ""]) as $b
        | .[$e.key] = ($e.value | if $b != null and $b > 10 then .stale = $b else . end) end)) as $rows1
| (reduce (CARRIED | to_entries[]) as $c ($rows1;
    if has($c.key) then . else .[$c.key] = {value: $c.value[0], unit: "", quiet: true, src: "carried: \($c.value[1])", carried: true, date: "", commit: ""} end)) as $rows
| [ROWS[] as [$row, $title, $target] | $rows[$row] as $r
   | if $r == null then {score: 0, row: $row, title: $title, value: "UNMEASURED", target: $target, gap: "", notes: ""}
     else (if $r.carried then null else $r | gap($row) end) as $g
     | (if $g then WEIGHT[$row[0:1]] * ($g | log2) else 0 end) as $score
     | ([ (if ($r.quiet | not) and ($r.load // {}).start != null then
             "under load \([$r.load.start, ($r.load.end // 0)] | max | fixed(0))",
             ($s.quiet[$row] | select(.) | "(quiet \(.value | show), \(.date[:10]))")
           else empty end),
          (if $r.carried then "CARRIED" else empty end),
          (if $r.stale then "STALE (\($r.stale) commits behind HEAD)" else empty end),
          (if $s.prev[$row] then ($s.prev[$row] | gap($row)) as $g0
             | if $g and $g0 and $g > $g0 * 1.1 then "REGRESSED (was \($s.prev[$row].value | show))"
               elif $g and $g0 and $g < $g0 / 1.1 then "improved (was \($s.prev[$row].value | show))" else empty end
           else empty end) ]) as $flag
     # r.get("unit", ""): a key that's there with null shows None
     | ("\($r.value | show) \(if $r | has("unit") then $r.unit | show else "" end)" | sub("^\\s+"; "") | sub("\\s+$"; "")) as $val
     | ("\(($r.date // "")[:16]) \(if $r | has("commit") then $r.commit | show else "" end)" | sub("^\\s+"; "") | sub("\\s+$"; "")) as $when
     | {score: $score, row: $row, title: $title, value: $val, target: $target, gap: (if $g then "x\($g | fixed(1))" else "-" end),
        notes: (($flag | join(" ")) + (if ($flag | length) > 0 then "  " else "" end) + "[\($r.src)\(if $when != "" then " " + $when else "" end)]")}
     end] as $table
| if $json == "1" then $table
  else ($table | sort_by(-.score)) as $sorted
    | ([$table[] | select(.value == "UNMEASURED") | .row]) as $um
    | ("row" | ljust(15)) + " " + ("gap" | rjust(6)) + " " + ("score" | rjust(5)) + "  " + ("value" | ljust(58)) + " target",
      ($sorted[] | (.row | ljust(15)) + " " + (.gap | rjust(6)) + " " + (.score | fixed(1) | rjust(5)) + "  " + (.value[:58] | ljust(58)) + " " + .target,
                   ("" | ljust(15)) + " " + ("" | rjust(6)) + " " + ("" | rjust(5)) + "  " + .title + "  " + .notes),
      "\n\(($table | length) - ($um | length)) of \($table | length) rows have a value" + (if ($um | length) > 0 then "; unmeasured: \($um | join(", "))" else "" end)
  end
