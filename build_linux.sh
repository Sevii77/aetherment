USERNAME="$(id -u -n)"
if [ "$1" = "client" ] && [ "$2" = "release" ]; then
	echo "client release"
	RUSTFLAGS="--remap-path-prefix /home/$USERNAME=" cargo build --release --target x86_64-unknown-linux-gnu --manifest-path=./client/Cargo.toml
	RUSTFLAGS="--remap-path-prefix /home/$USERNAME=" cargo build --release --target x86_64-pc-windows-gnu --manifest-path=./client/Cargo.toml
elif [ "$1" = "client" ]; then
	echo "client dev"
	cargo run --manifest-path=./client/Cargo.toml
elif [ "$1" = "plugin" ] && [ "$2" = "release" ]; then
	echo "plugin release"
	RUSTFLAGS="--remap-path-prefix /home/$USERNAME=" cargo build --lib --release --target x86_64-pc-windows-gnu --manifest-path=./plugin/Cargo.toml
	dotnet build ./plugin/plugin/Aetherment.csproj -c Release
elif [ "$1" = "plugin" ]; then
	echo "plugin dev"
	RUSTFLAGS="--remap-path-prefix /home/$USERNAME=" cargo build --lib --target x86_64-pc-windows-gnu --manifest-path=./plugin/Cargo.toml
	dotnet build ./plugin/plugin/Aetherment.csproj -c Debug
else
	echo "invalid target"
fi