#!/bin/bash
set -e

echo "Installing dependencies for Jarvis on Linux..."

# Install Ollama
if ! command -v ollama &> /dev/null; then
    echo "Installing Ollama..."
    curl -fsSL https://ollama.com/install.sh | sh
else
    echo "Ollama is already installed."
fi

# Install PostgreSQL and pgvector
echo "Installing PostgreSQL and pgvector..."
if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y postgresql postgresql-contrib
    
    # Install pgvector (depends on postgres version)
    PG_VERSION=$(psql --version | grep -oE '[0-9]+' | head -n 1)
    sudo apt-get install -y "postgresql-$PG_VERSION-pgvector" || {
        echo "Could not find pre-packaged pgvector. Attempting to build from source..."
        sudo apt-get install -y postgresql-server-dev-all build-essential
        git clone --branch v0.7.0 https://github.com/pgvector/pgvector.git /tmp/pgvector
        cd /tmp/pgvector
        make
        sudo make install
    }
elif command -v dnf &> /dev/null; then
    sudo dnf install -y postgresql-server postgresql-contrib
    # pgvector on Fedora/RHEL usually needs manual build or specific repo
    echo "Please ensure pgvector is installed for your PostgreSQL instance."
else
    echo "Unsupported package manager. Please install PostgreSQL and pgvector manually."
fi

echo "Setup complete!"
echo "Next steps:"
echo "1. Start PostgreSQL: sudo service postgresql start"
echo "2. Create a database: sudo -u postgres createdb jarvis"
echo "3. Enable pgvector: sudo -u postgres psql -d jarvis -c 'CREATE EXTENSION vector;'"
echo "4. Pull the LLM model: ollama pull llama3"
echo "5. Set DATABASE_URL environment variable: export DATABASE_URL=postgres://postgres@localhost/jarvis"
