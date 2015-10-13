.PHONY: release debug docker deb-pkg clean all

UPSTREAM_TARBALL=sota-client_1.0.orig.tar.gz
SRCS := $(wildcard src/*.rs)
SRCS += Cargo.toml

target/release/sota_client: $(SRCS)
	cargo build --release

target/debug/sota_client: $(SRCS)
	cargo build

docker/sota_client: target/release/sota_client
	cp target/release/sota_client docker

docker: docker/sota_client
	docker build -t advancedtelematic/sota-client docker

deb-pkg: target/release/sota_client
	tar czf pkg/$(UPSTREAM_TARBALL) sota_client.service target/release/sota_client client.toml
	cd pkg/sota-client_1.0 && tar xzf ../$(UPSTREAM_TARBALL) && debuild

clean:
	rm -fr pkg/client.toml
	rm -fr pkg/sota_client
	rm -fr pkg/sota-client_1.0/*.changes
	rm -fr pkg/*.tar.gz
	rm -fr pkg/*.tar.xz
	rm -fr pkg/*.dsc
	rm -fr pkg/*.build
	rm -fr pkg/*.deb
	rm -fr pkg/sota-client_1.0/pkg
	rm -fr pkg/sota-client_1.0/debian/sota-client

# aliases
debug: target/debug/sota_client
release: target/release/sota_client
all: docker
