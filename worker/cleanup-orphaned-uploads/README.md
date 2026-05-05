# Cleanup Orphaned Uploads Worker

This worker is responsible for cleaning up abandoned or orphaned multipart uploads in the Cloudflare R2 bucket. Since
the file-sharing service uses multipart uploads for large files, some uploads may never be completed or aborted if a
user closes their browser or a network error occurs.

## Overview

The worker periodically (or when triggered) scans the S3 bucket for active multipart uploads under the `uploads/`
prefix. Any upload that was initiated more than **6 hours ago** is considered orphaned and is automatically aborted.
Aborting an orphaned multipart upload ensures that any partially uploaded chunks are deleted, freeing up storage space
and reducing costs.

## Configuration

The worker is configured via environment variables. It uses the standard AWS SDK environment variables for credentials
and region, plus a specific one for the bucket name.

| Variable                | Description                                                                 |
|-------------------------|-----------------------------------------------------------------------------|
| `S3_BUCKET`             | The name of the R2/S3 bucket to scan.                                       |
| `AWS_ACCESS_KEY_ID`     | R2/S3 Access Key.                                                           |
| `AWS_SECRET_ACCESS_KEY` | R2/S3 Secret Key.                                                           |
| `AWS_REGION`            | Usually `auto` for Cloudflare R2.                                           |
| `AWS_ENDPOINT_URL`      | The R2 endpoint URL (e.g., `https://<accountid>.r2.cloudflarestorage.com`). |
| `RUST_LOG`              | Logging level (e.g., `info`, `debug`).                                      |

## Development

### Prerequisites

- Rust toolchain (Edition 2024)

### Running Locally

1. Create a `.env` file in this directory or set the environment variables manually.
2. Run the worker:

```bash
cargo run
```

## Docker

A `Dockerfile` is provided for containerized deployment.

### Build

```bash
docker build -t cleanup-orphaned-uploads .
```

### Run

```bash
docker run --env-file .env cleanup-orphaned-uploads
```
