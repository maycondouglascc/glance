PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
APPDIR ?= $(PREFIX)/share/applications
METAINFO ?= $(PREFIX)/share/metainfo

.PHONY: all build release test clean install uninstall

all: release

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --workspace

install: release
	install -d $(BINDIR) $(APPDIR) $(METAINFO)
	install -m 755 target/release/glance $(BINDIR)/glance
	install -m 755 target/release/glance-tree $(BINDIR)/glance-tree
	install -m 644 data/io.github.maycon.Glance.desktop $(APPDIR)/io.github.maycon.Glance.desktop
	install -m 644 data/io.github.maycon.Glance.metainfo.xml $(METAINFO)/io.github.maycon.Glance.metainfo.xml
	@echo "Glance installed successfully to $(BINDIR)/glance."
	@echo "You can launch it from your app launcher or by running: glance"

uninstall:
	rm -f $(BINDIR)/glance $(BINDIR)/glance-tree
	rm -f $(APPDIR)/io.github.maycon.Glance.desktop
	rm -f $(METAINFO)/io.github.maycon.Glance.metainfo.xml
	@echo "Glance uninstalled."

clean:
	cargo clean
