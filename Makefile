

build: fmt
	cargo build

fmt:
	cargo fmt

release:
	cargo build --release

gshare: release
	bunzip2 -kc ./traces/fp_1.bz2   | ./target/release/cse240a --predictor gshare:13
	bunzip2 -kc ./traces/fp_2.bz2   | ./target/release/cse240a --predictor gshare:13
	bunzip2 -kc ./traces/int_1.bz2  | ./target/release/cse240a --predictor gshare:13
	bunzip2 -kc ./traces/int_2.bz2  | ./target/release/cse240a --predictor gshare:13
	bunzip2 -kc ./traces/mm_1.bz2   | ./target/release/cse240a --predictor gshare:13
	bunzip2 -kc ./traces/mm_2.bz2   | ./target/release/cse240a --predictor gshare:13

tournament: release
	bunzip2 -kc ./traces/fp_1.bz2   | ./target/release/cse240a --predictor tournament:9:10:10
	bunzip2 -kc ./traces/fp_2.bz2   | ./target/release/cse240a --predictor tournament:9:10:10
	bunzip2 -kc ./traces/int_1.bz2  | ./target/release/cse240a --predictor tournament:9:10:10
	bunzip2 -kc ./traces/int_2.bz2  | ./target/release/cse240a --predictor tournament:9:10:10
	bunzip2 -kc ./traces/mm_1.bz2   | ./target/release/cse240a --predictor tournament:9:10:10
	bunzip2 -kc ./traces/mm_2.bz2   | ./target/release/cse240a --predictor tournament:9:10:10

custom: release
	bunzip2 -kc ./traces/fp_1.bz2   | ./target/release/cse240a --predictor custom:34:305:79
	bunzip2 -kc ./traces/fp_2.bz2   | ./target/release/cse240a --predictor custom:34:305:79
	bunzip2 -kc ./traces/int_1.bz2  | ./target/release/cse240a --predictor custom:34:305:79
	bunzip2 -kc ./traces/int_2.bz2  | ./target/release/cse240a --predictor custom:34:305:79
	bunzip2 -kc ./traces/mm_1.bz2   | ./target/release/cse240a --predictor custom:34:305:79
	bunzip2 -kc ./traces/mm_2.bz2   | ./target/release/cse240a --predictor custom:34:305:79
