COLOR ?= always # Valid COLOR options: {always, auto, never}
CARGO ?= cargo --color $(COLOR)
WATCH ?= cargo watch -c -x
RUSTFLAGS ?= -Awarnings

.PHONY: all bench build check clean doc install publish run test update

all: build

bench:
	@$(CARGO) bench

build:
	@$(CARGO) build

check:
	RUSTFLAGS=$(RUSTFLAGS) @$(CARGO) check

clean:
	@$(CARGO) clean

doc:
	@$(CARGO) rustdoc --bin book-summary --open -- --document-private-items
	@xdg-open target/doc/book-summary/index.html

install: build
	@$(CARGO) install --path=. --force

publish:
	@$(CARGO) publish

run: build
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) run -- -n examples/book

test:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) test

watch:
	@$(WATCH) check -x test
