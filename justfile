set windows-shell := ["cmd.exe", "/c"]

build configuration="debug": (build-rust configuration) (build-swift configuration)

build-rust flags="":
    cargo build {{ if flags == "debug" { "" } else { "--" + flags } }} --manifest-path rust/Cargo.toml

build-swift configuration="debug":
    swift build --configuration {{ configuration }} --package-path swift/

[unix]
export preset configuration="debug": (build configuration)
    mkdir -p godot/export/{{preset}}
    godot --path godot/ --export-{{configuration}} {{preset}} --headless

[windows]
export preset configuration="debug": (build configuration)
    mkdir godot/export/{{preset}}
    godot --path godot/ --export-{{configuration}} {{preset}} --headless


clean:
    cargo clean --manifest-path rust/Cargo.toml
    swift package clean --package-path swift/
