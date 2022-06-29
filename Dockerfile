FROM rust:alpine as build

ENV RUSTFLAGS="-C target-feature=-crt-static"
WORKDIR /usr/src/saysthbot
COPY . .
RUN rustup default nightly && cargo build --release

FROM alpine

RUN apk add --no-cache ca-certificates openssl
ENV TGBOT_TOKEN="" DATABASE_URI="" WRAPPER=""
CMD ["-c", "${WRAPPER} ./saysthbot-reborn ${OPTIONS}"]
ENTRYPOINT [ "/bin/sh" ]

COPY --from=build /usr/src/saysthbot/target/release/saysthbot-reborn ./
