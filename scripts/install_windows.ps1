Write-Host "Installing dependencies for Jarvis on Windows..."

# Install Ollama
if (!(Get-Command ollama -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Ollama via winget..."
    winget install ollama
} else {
    Write-Host "Ollama is already installed."
}

# Recommendation for PostgreSQL/pgvector
Write-Host ""
Write-Host "For PostgreSQL with pgvector on Windows, we recommend using Docker:"
Write-Host "docker run --name jarvis-db -e POSTGRES_PASSWORD=password -p 5432:5432 -d ankane/pgvector"
Write-Host ""
Write-Host "Alternatively, you can install PostgreSQL manually and follow pgvector's build instructions for Windows."

Write-Host "Next steps:"
Write-Host "1. Start Ollama and pull the model: ollama pull llama3"
Write-Host "2. Ensure PostgreSQL is running with pgvector enabled."
Write-Host "3. Create a 'jarvis' database."
Write-Host "4. Set the DATABASE_URL environment variable: `$env:DATABASE_URL='postgres://postgres:password@localhost/jarvis'`"
