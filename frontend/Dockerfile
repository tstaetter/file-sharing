FROM denoland/deno:2

WORKDIR /app

# Install dependencies (layered for caching)
COPY deno.json deno.lock* ./
RUN deno install --frozen

# Copy source code
COPY . .

EXPOSE 5173

CMD ["deno", "task", "dev", "--", "--host", "0.0.0.0"]
