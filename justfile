set windows-shell := ["cmd.exe", "/c"]

build: build-rust build-swift

build-rust:
    cargo build --manifest-path rust/Cargo.toml

build-swift:
    swift build --configuration debug --package-path swift/

clean:
    cargo clean --manifest-path rust/Cargo.toml
    swift package clean --package-path swift/
    
