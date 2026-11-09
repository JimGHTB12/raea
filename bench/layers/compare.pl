#!/usr/bin/perl
# bench/layers/compare.sh's comparison: NAME.layers on stdin, the oracle folder and the IoU floor as arguments.
use strict; my ($oracle, $min) = @ARGV; binmode STDIN; local $/; my $d = <STDIN>;
substr($d, 0, 8) eq "RAEALAYR" or die "not a layers file\n";
my $n = unpack("V", substr($d, 8, 4)); my $h = substr($d, 12, $n); my $o = 12 + $n;
# the header, read with no JSON module: the mask size, the moments, and each layer's key and parent
my ($mw, $mh) = $h =~ /"mask":\{"w":(\d+),"h":(\d+)/ or die "no mask size\n";
my @at = $h =~ /"at":\[([\d,]+)\]/ ? split(/,/, $1) : ();
my (@key, @parent);
while ($h =~ /\{"key":"((?:[^"\\]|\\.)*)","kind":"[a-z]+","name":"(?:[^"\\]|\\.)*","parent":(null|"(?:[^"\\]|\\.)*")/g) {
  push @key, $1; my $p = $2; $p =~ s/^"|"$//g; push @parent, $p eq "null" ? undef : $p;
}
my %index; $index{$key[$_]} = $_ + 1 for 0..$#key;
# each value's ancestors-or-self, as values
my @up = ([]); for my $v (1..@key) { my @c = ($v); my $p = $parent[$v - 1]; while (defined $p && $index{$p}) { push @c, $index{$p}; $p = $parent[$index{$p} - 1] } $up[$v] = \@c }
my (@frames, @prev);
my $count = $h =~ /"frames":(\d+),"width"/ ? $1 : die "no frame count\n";
for (1..$count) {
  my $runs = unpack("V", substr($d, $o, 4)); $o += 4; my @v;
  for (1..$runs) { my ($v, $c) = unpack("vv", substr($d, $o, 4)); $o += 4; push @v, ($v) x $c }
  @v = @prev unless $runs; @prev = @v; push @frames, [@v];
}
@at = (0..$count - 1) unless @at;
open my $k, "<", "$oracle/keys.txt" or die; my $fail = 0;
for (split /\n/, do { local $/; <$k> }) {
  my ($i, $want) = split / /, $_, 2; my $v = $index{$want};
  my @ious;
  for my $m (0..$#at) {
    my $pgm = "$oracle/$at[$m]/$i.pgm"; -f $pgm or next;
    open my $p, "<", $pgm or die; binmode $p; my $raw = do { local $/; <$p> };
    $raw =~ s/\AP5\s+(\d+)\s+(\d+)\s+(\d+)\s//s or die "$pgm: not a P5 PGM\n"; my $max = $3; ($1 == $mw && $2 == $mh) or die "$pgm: ${1}x$2, the layers are ${mw}x$mh\n";
    my @want = unpack("C*", $raw); my ($both, $either) = (0, 0);
    for my $px (0..$#want) {
      my $got = $v && grep { $_ == $v } @{$up[$frames[$m][$px]]};
      my $w = $want[$px] * 2 > $max;
      $both++ if $got && $w; $either++ if $got || $w;
      if ($ENV{LAYERS_WHY} && $ENV{LAYERS_WHY} eq "$i/$at[$m]" && ($got xor $w)) { printf "  %s at %d,%d (value there %d)\n", $got ? "only the layers" : "only the oracle", $px % $mw, int($px / $mw), $frames[$m][$px] }
    }
    push @ious, $either ? $both / $either : 1;
  }
  my $worst = (sort { $a <=> $b } @ious)[0] // 0;
  my $ok = defined $v && $worst >= $min;
  $fail ||= !$ok;
  printf "%s layer %s (%s): IoU %s%s\n", $ok ? "ok  " : "FAIL", $i, $want, join(" ", map { sprintf "%.3f", $_ } @ious), defined $v ? "" : ", not in the layers";
}
exit($fail ? 1 : 0);
