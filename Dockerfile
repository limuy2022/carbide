FROM rust:latest AS cross-compile

FROM debian:bookworm-slim

RUN groupadd -r carbide && \
    useradd -r -g carbide carbide && \
    mkdir -p /carbide/data && \
    chown -R carbide:carbide /carbide && \
    rm -rf /var/lib/apt/lists/*

USER carbide
VOLUME /carbide/data
ENV HOME=/carbide/data

COPY . /carbide

RUN /carbide/carbide --version

ENTRYPOINT ["/carbide/carbide", "--no-sandbox", "--disable-dev-shm-usage", "--user-data-dir=/carbide/data"]
