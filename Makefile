rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
rust_files=$(call rwildcard,src,*.rs)

test: $(rust-files)
	cargo test

build: $(rust-files)
	cargo build

tarpaulin-report.%: $(rust_files)
	cargo tarpaulin --skip-clean --test --out $*

cover: tarpaulin-report.html

view-cover: tarpaulin-report.html
	open $<
	
clean-tarpaulin:
	rm -f tarpaulin-report.*

clean: clean-tarpaulin
	cargo clean

