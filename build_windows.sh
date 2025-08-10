# sorry not sorry for making this a bash file, bat is fucking horrible and i dont want to touch it
DOTNET_CLI_TELEMETRY_OPTOUT="1"

if [ "$1" = "client" ] && [ "$2" = "release" ]; then
	echo "client release"
	cargo build --release --target x86_64-pc-windows-msvc --manifest-path=./client/Cargo.toml
elif [ "$1" = "client" ]; then
	echo "client dev"
	cargo run --manifest-path=./client/Cargo.toml
elif [ "$1" = "plugin" ] && [ "$2" = "release" ]; then
	echo "plugin release"
	cargo build --lib --release --target x86_64-pc-windows-msvc --manifest-path=./plugin/Cargo.toml
	dotnet build ./plugin/plugin/Aetherment.csproj -c Release
elif [ "$1" = "plugin" ]; then
	echo "plugin dev"
	cargo  build --lib --target x86_64-pc-windows-msvc --manifest-path=./plugin/Cargo.toml
	dotnet build ./plugin/plugin/Aetherment.csproj -c Debug
else
	echo "invalid target"
fi