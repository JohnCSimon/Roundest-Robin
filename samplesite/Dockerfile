FROM python:3.11-slim

WORKDIR /app

# Copy static files
COPY . /app/static

# Expose port
EXPOSE 8000

# Serve static files using Python's built-in HTTP server
CMD ["python", "-m", "http.server", "8000", "--directory", "static"]