mkdir images -p
cargo run --release hilbert    11 -s 2048 -t 0     -c 2   -o images/hilbert.png
cargo run --release moore      11 -s 2048 -t 0     -c bw3 -o images/moore.png
cargo run --release zorder     11 -s 2048 -t 0     -c 4   -o images/z-order.png
cargo run --release dragon     21 -s 2048 -t 0 -c cet-l19 -o images/dragon.png     --bg 525
cargo run --release gosper      8 -s 2048 -t 0     -c h   -o images/gosper.png     --bg b0b4b8
cargo run --release sierpinski 13 -s 2048 -t 0 -c cet-l17 -o images/sierpinski.png --bg 000a00
cargo run --release square     11 -s 2048 -t 0     -c h   -o images/square.png
cargo run --release koch        8 -s 2048 -t 0 -c cet-117 -o images/koch.png       --bg 000a00
cargo run --release triangle   11 -s 2048 -t 0     -c 3   -o images/triangle.png   --bg fff
cargo run --release s-curve     5 -s 2048 -t 0.275 -c ry  -o images/s.png          --bg ddc
cargo run --release wunderlich  3 -s 2048 -t 0.75  -c m   -o images/wunderlich.png --bg 222
cargo run --release arioni     10 -s 2048 -t 0     -c o6  -o images/arioni.png
cargo run --release steemann    3 -s 2048 -t 0.45  -c cet-l08 -o images/steemann.png --bg 111
cargo run --release fivefold   11 -s 2048 -t 0     -c h   -o images/fivefold.png
