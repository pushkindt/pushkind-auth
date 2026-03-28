check:
	cargo fmt --all
	cargo clippy --all-features --tests -- -Dwarnings
	cargo test --all-features
	cargo test --all-features --test e2e -- --ignored

coverage:
	cargo tarpaulin --all-features --all-targets --out Html
	wslview tarpaulin-report.html
