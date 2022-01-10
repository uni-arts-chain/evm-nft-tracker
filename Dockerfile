FROM rust:1.54 as builder

WORKDIR /usr/src/evm-nft-tracker
COPY . .

RUN cargo b --release

#ENTRYPOINT ["./run.sh"]

FROM ubuntu:20.04
RUN apt update && apt install -y libssl-dev ca-certificates
COPY --from=builder /usr/src/evm-nft-tracker/target/release/ethereum-nft-tracker /usr/local/bin
COPY docker-entrypoint.sh docker-entrypoint.sh
ENTRYPOINT ["./docker-entrypoint.sh"]
