#!/bin/bash

echo "Mrglglglgl! Instalando Murlang!"

# Verifica se o diretório bin existe
mkdir -p "$(dirname "$0")/../bin"

# Compila o executável
echo "Mrglgrgl... Compilando executável..."
pushd "$(dirname "$0")/.." > /dev/null
cargo build --release
if [ $? -ne 0 ]; then
    echo "Aaaaaughibbrgubugbugrguburglegrrr! Erro na compilação!"
    exit 1
fi
cp "target/release/mur_lang" "bin/murlang"
chmod +x "bin/murlang"
popd > /dev/null

# Cria o script wrapper
echo "Mrglmrgl... Criando wrapper..."
cat > "$(dirname "$0")/../bin/mrgl" << 'EOL'
#!/bin/bash
# Murlang Runner - Mrglglgl!

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export MURLANG_HOME="$(dirname "$SCRIPT_DIR")"
export MURLANG_VERSION="1.0.0"

if [ "$1" = "run" ]; then
    if [ -z "$2" ]; then
        echo "Mrglgrgl! Especifique um arquivo .mur para executar!"
        exit 1
    fi
    "$MURLANG_HOME/bin/murlang" "$2"
    exit $?
fi

if [ "$1" = "version" ]; then
    echo "Mrglglgl! Murlang versão $MURLANG_VERSION"
    exit 0
fi

if [ "$1" = "help" ]; then
    echo "Mrglglglgl! Comandos disponíveis:"
    echo "  mrgl run <arquivo.mur>    - Executa um programa Murlang"
    echo "  mrgl version          - Mostra a versão do Murlang"
    echo "  mrgl help             - Mostra esta ajuda"
    exit 0
fi

echo "Mrglglgl? Comando desconhecido. Use 'mrgl help' para ajuda."
exit 1
EOL

chmod +x "$(dirname "$0")/../bin/mrgl"

# Cria um script de ambiente
echo "Mrglglgl... Criando script de ambiente..."
cat > "$(dirname "$0")/../bin/activate" << 'EOL'
#!/bin/bash
# Murlang Environment Setup

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export MURLANG_HOME="$(dirname "$SCRIPT_DIR")"
export PATH="$MURLANG_HOME/bin:$PATH"

echo "Mrglglglgl! Ambiente Murlang ativado!"
echo "Use 'mrgl help' para ver os comandos disponíveis."
EOL

chmod +x "$(dirname "$0")/../bin/activate"

echo "Mrglglglgl! Instalação concluída!"
echo ""
echo "Para usar o Murlang, adicione $(dirname "$0")/../bin ao seu PATH ou execute:"
echo "    source $(dirname "$0")/../bin/activate"
echo ""
echo "Depois, você pode executar:"
echo "    mrgl run seu_arquivo.mur"
echo ""
echo "Glrglmrgl! Boa programação!" 