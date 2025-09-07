mkdir images -p
cargo run --release hilbert    11 -s 2048 -t 0     -c 2   -o images/hilbert.png
cargo run --release moore      11 -s 2048 -t 0     -c bw3 -o images/moore.png
cargo run --release zorder     11 -s 2048 -t 0     -c 4   -o images/z-order.png
cargo run --release dragon     21 -s 2048 -t 0     -c 8   -o images/dragon.png
cargo run --release gosper      8 -s 2048 -t 0     -c h   -o images/gosper.png
cargo run --release sierpinski 13 -s 2048 -t 0     -c b   -o images/sierpinski.png
cargo run --release square     11 -s 2048 -t 0     -c 6   -o images/square.png
cargo run --release square     11 -s 2048 -t 0     -c 8   -o images/square-2.png
cargo run --release koch        8 -s 2048 -t 0     -c 2   -o images/koch.png
cargo run --release triangle   11 -s 2048 -t 0     -c 3   -o images/triangle.png
cargo run --release s-curve     5 -s 2048 -t 0.275 -c rgy -o images/s.png
cargo run --release wunderlich  7 -s 2048 -t 0     -c m   -o images/wunderlich.png
cargo run --release arioni     10 -s 2048 -t 0     -c o6  -o images/arioni.png
cargo run --release steemann    8 -s 2048 -t 0     -c 7   -o images/steemann.png
cargo run --release fivefold   11 -s 2048 -t 0     -c h   -o images/fivefold.png
