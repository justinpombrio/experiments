run() {
    cargo run --release -- -s 2048 "$@"
}

mkdir images -p
run moore               11 -t 0     -c 3                   -o images/moore.png
run zorder              11 -t 0     -c 4                   -o images/z-order.png
run dragon              22 -t 0     -c cet-l19 --bg 636    -o images/dragon.png   --fg 525 -a 0.075 
run gosper               8 -t 0     -c h       --bg ddd    -o images/gosper.png
run gosper               8 -t 0     -c h       --bg eee    -o images/gosper.png --fg 080808 
run sierpinski-triangle 13 -t 0     -c cet-l17 --bg 000a00 -o images/sierpinski-triangle.png
run sierpinski-curve    11 -t 0     -c h                   -o images/sierpinski-curve.png
run koch                 6 -t 0.75  -c b       --bg ddd    -o images/koch.png     --fill abc -a 0.25
run triangle            11 -t 0     -c 6       --bg fff    -o images/triangle.png
run s-curve              5 -t 0.275 -c ry      --bg 181812 -o images/s.png        --style curvy
run aztec                3 -t 0.65  -c m       --bg 181818 -o images/aztec.png
run arioni               5 -t 0.7   -c o6      --bg 222    -o images/arioni.png   -a -0.078
run steemann             3 -t 0.45  -c cet-l08 --bg 111    -o images/steemann.png --style curvy
run fivefold            10 -t 0     -c h       --bg fff    -o images/fivefold.png -a 0.6667 -s 1024
