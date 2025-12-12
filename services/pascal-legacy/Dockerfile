FROM debian:12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    fp-compiler \
    postgresql-client \
    ca-certificates \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY legacy.pas ./
COPY csv_to_xlsx.py ./
COPY run.sh /usr/local/bin/run_legacy.sh
RUN chmod +x /usr/local/bin/run_legacy.sh && \
    chmod +x csv_to_xlsx.py && \
    pip3 install --no-cache-dir openpyxl

CMD ["/usr/local/bin/run_legacy.sh"]
