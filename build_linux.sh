if [ "$1" = "client" ] && [ "$2" = "release" ]; then
	echo "client release"
	RUSTFLAGS="-l dylib=stdc++" cargo build --release --target x86_64-unknown-linux-gnu --manifest-path=./client/Cargo.toml
elif [ "$1" = "client" ]; then
	echo "client dev"
	RUSTFLAGS="-l dylib=stdc++" cargo run --manifest-path=./client/Cargo.toml
elif [ "$1" = "plugin" ] && [ "$2" = "release" ]; then
	echo "plugin release"
	cargo xwin build --lib --release --target x86_64-pc-windows-msvc --manifest-path=./plugin/Cargo.toml
	# cp ./lib/bcryptprimitives.dll ./target/x86_64-pc-windows-msvc/release/bcryptprimitives.dll
	dotnet build ./plugin/plugin/Aetherment.csproj -c Release
elif [ "$1" = "plugin" ]; then
	echo "plugin dev"
	cargo xwin build --lib --target x86_64-pc-windows-msvc --manifest-path=./plugin/Cargo.toml
	# cp ./lib/bcryptprimitives.dll ./target/x86_64-pc-windows-msvc/debug/bcryptprimitives.dll
	dotnet build ./plugin/plugin/Aetherment.csproj -c Debug
else
	echo "invalid target"
fi