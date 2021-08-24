FROM rust:1.54

WORKDIR /usr/src/evm-nft-tracker
COPY . .

RUN cargo b --release

ENTRYPOINT ["./run.sh"]
