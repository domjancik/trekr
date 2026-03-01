.PHONY: setup run run-demo run-empty check fmt capture-ui review-ui ui-review

ABLETON_LINK_HEADER := vendor/ableton-link/include/ableton/Link.hpp

$(ABLETON_LINK_HEADER):
	git submodule update --init --recursive vendor/ableton-link

setup: $(ABLETON_LINK_HEADER)

run: setup
	cargo run

run-demo: setup
	cargo run -- --state-mode demo

run-empty: setup
	cargo run -- --state-mode empty

check: setup
	cargo check

fmt:
	cargo fmt

capture-ui: setup
	powershell -ExecutionPolicy Bypass -File .\scripts\capture-ui-screens.ps1 -StateMode demo

review-ui: setup
	powershell -ExecutionPolicy Bypass -File .\scripts\review-ui-screens.ps1 -StateMode demo

ui-review: setup
	powershell -ExecutionPolicy Bypass -File .\scripts\run-ui-review.ps1 -StateMode demo
