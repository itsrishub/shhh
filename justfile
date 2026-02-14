binary := "shhh"
target_dir := "target/release"
install_path := "/usr/local/bin"

default: build install

build:
    cargo build --release

install:
    sudo cp {{target_dir}}/{{binary}} {{install_path}}/

clean:
    cargo clean