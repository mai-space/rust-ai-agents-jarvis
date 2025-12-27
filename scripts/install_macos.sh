#!/bin/bash
set -e

echo "Installing dependencies for Jarvis on macOS..."

# Check for Homebrew
if ! command -v brew &> /dev/null; then
    echo "Homebrew not found. Please install it from https://brew.sh/"
    exit 1
fi

# Install Ollama
if ! command -v ollama &> /dev/null; then
    echo "Installing Ollama..."
    brew install ollama
else
    echo "Ollama is already installed."
fi

# Install PostgreSQL
echo "Installing PostgreSQL..."
if ! command -v psql &> /dev/null; then
    brew install postgresql@14
    brew link postgresql@14
else
    echo "PostgreSQL is already installed."
fi

# Install pgvector
echo "Installing pgvector..."
brew install pgvector

echo "Setup complete!"
echo "Next steps:"
echo "1. Start PostgreSQL: brew services start postgresql@14"
echo "2. Create a database: createdb jarvis"
echo "3. Enable pgvector: psql -d jarvis -c 'CREATE EXTENSION vector;'"
echo "4. Pull the LLM model: ollama pull llama3"
echo "5. Set DATABASE_URL environment variable: export DATABASE_URL=postgres://localhost/jarvis"
