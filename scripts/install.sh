#!/bin/bash

cat "$(dirname "$0")/banner.txt"
echo ""

mkdir -p "$(dirname "$0")/../bin"

echo "Mrglgrgl... Compiling executable..."
pushd "$(dirname "$0")/.." > /dev/null
cargo build --release
if [ $? -ne 0 ]; then
    echo "Aaaaaughibbrgubugbugrguburglegrrr! Error compiling!"
    exit 1
fi
cp "target/release/mur_lang" "bin/murlang"
chmod +x "bin/murlang"
popd > /dev/null

echo "Creating wrapper..."
cat > "$(dirname "$0")/../bin/mrgl" << 'EOL'
#!/bin/bash
# Murlang Runner - Mrglglgl!

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export MURLANG_HOME="$(dirname "$SCRIPT_DIR")"

if [ "$1" = "run" ]; then
    if [ -z "$2" ]; then
        echo "Mrglgrgl! Specify a .mur file to execute!"
        exit 1
    fi
    "$MURLANG_HOME/bin/murlang" "$2"
    exit $?
fi

if [ "$1" = "help" ]; then
    "$MURLANG_HOME/bin/murlang" help
    exit 0
fi

if [ "$1" = "--version" ] || [ "$1" = "-V" ]; then
    "$MURLANG_HOME/bin/murlang" --version
    exit 0
fi

echo "Mrglglgl? Unknown command. Use 'mrgl help' for help."
exit 1
EOL

chmod +x "$(dirname "$0")/../bin/mrgl"

echo "Adding to PATH..."
BIN_PATH="$(dirname "$0")/../bin"
echo "export PATH=\"$BIN_PATH:\$PATH\"" >> ~/.bashrc
echo "export MURLANG_HOME=\"$(dirname "$BIN_PATH")\"" >> ~/.bashrc

source ~/.bashrc

echo ""
echo "Mglrmglmglmgl! Installation completed!"
echo ""
echo "To use Murlang, try:"
echo "    mrgl help"
echo ""
echo "If the command is not recognized, try running:"
echo "    $BIN_PATH/mrgl help"
echo ""
echo "Aaaaaughibbrgubugbugrguburgle!" 