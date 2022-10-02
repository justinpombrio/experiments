mkdir images -p
cargo run --release hilbert    11 -t 0 -s 2048 -c 2  -o images/hilbert.png
cargo run --release hilbert    11 -t 0 -s 2048 -c bw -o images/hilbert-bw.png
cargo run --release zorder     11 -t 0 -s 2048 -c 4  -o images/z-order.png
cargo run --release dragon     21 -t 0 -s 2048 -c 8  -o images/dragon.png
cargo run --release gosper     8  -t 0 -s 2048 -c 9  -o images/gosper.png
cargo run --release peano      7  -t 0 -s 2048 -c 3  -o images/peano.png
cargo run --release sierpinski 11 -t 0 -s 2048 -c 6  -o images/sierpinski.png
cargo run --release sierpinski 11 -t 0 -s 2048 -c 8  -o images/sierpinski-2.png
cargo run --release koch       8  -t 0 -s 2048 -c 2  -o images/koch.png
cargo run --release triangle   11 -t 0 -s 2048 -c 2  -o images/triangle.png
cargo run --release fivefold   11 -t 0 -s 2048 -c 7  -o images/fivefold.png
