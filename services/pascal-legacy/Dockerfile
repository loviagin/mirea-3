FROM debian:12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    fp-compiler \
    fpc \
    fpc-source \
    postgresql-client \
    ca-certificates \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Создаем виртуальное окружение для Python
RUN python3 -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Устанавливаем openpyxl в виртуальное окружение
RUN pip3 install --no-cache-dir openpyxl

COPY legacy.pas ./
COPY csv_to_xlsx.py ./
COPY run.sh /usr/local/bin/run_legacy.sh
RUN chmod +x /usr/local/bin/run_legacy.sh && \
    chmod +x csv_to_xlsx.py

CMD ["/usr/local/bin/run_legacy.sh"]
