set terminal pngcairo size 960,540
set output "winrates.png"

set title "Win Rate by Strategy"
set ylabel "win rate"
set yrange [0:1]
set style data histogram
set style histogram clustered gap 1
set style fill solid 0.85 border -1
set boxwidth 0.8
set grid ytics
set key off

plot "data.txt" using 4:xtic(1)
