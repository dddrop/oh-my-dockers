# Chroma Vector Database Configuration

This directory is mounted to the Chroma container for initialization and configuration.

## Authentication

Chroma uses basic authentication configured via environment variables:

```env
CHROMA_SERVER_AUTH_CREDENTIALS=admin:admin
CHROMA_SERVER_AUTH_PROVIDER=chromadb.auth.basic_authn.BasicAuthenticationServerProvider
```

You can change these in your project's `.env` file.

## Data Persistence

Chroma data is persisted in a Docker volume. The database files are stored at `/chroma/chroma` in the container.

## Accessing Chroma

Chroma will be available at:
- Internal (from other containers): `http://chroma:8000`
- External (via Caddy): `https://chroma.daily.local`

## API Documentation

Once running, you can access the API documentation at:
- `https://chroma.daily.local/docs`

## Example Usage

```python
import chromadb

# Create client
client = chromadb.HttpClient(
    host="chroma.daily.local",
    port=443,
    ssl=True,
    headers={"Authorization": "Basic YWRtaW46YWRtaW4="}  # base64 of admin:admin
)

# Get or create a collection
collection = client.get_or_create_collection("my_collection")

# Add documents
collection.add(
    documents=["This is a document", "This is another document"],
    metadatas=[{"source": "my_source"}, {"source": "my_source"}],
    ids=["id1", "id2"]
)

# Query
results = collection.query(
    query_texts=["This is a query document"],
    n_results=2
)
```

