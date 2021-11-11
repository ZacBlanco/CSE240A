

build: fmt
	cargo build

fmt:
	cargo fmt

profile:
	bunzip2 -kc ./traces/fp_1.bz2   | time cargo run --release -- --predictor gshare:32 2> /dev/null
	bunzip2 -kc ./traces/fp_2.bz2   | time cargo run --release -- --predictor gshare:32 2> /dev/null
	bunzip2 -kc ./traces/int_1.bz2  | time cargo run --release -- --predictor gshare:32 2> /dev/null
	bunzip2 -kc ./traces/int_2.bz2  | time cargo run --release -- --predictor gshare:32 2> /dev/null
	bunzip2 -kc ./traces/mm_1.bz2   | time cargo run --release -- --predictor gshare:32 2> /dev/null
	bunzip2 -kc ./traces/mm_2.bz2   | time cargo run --release -- --predictor gshare:32 2> /dev/null
