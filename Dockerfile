FROM alpine as build

ENV RUSTFLAGS="-C target-feature=-crt-static"

COPY ./ /saysthbot

RUN apk add --no-cache rust cargo && \
    cd /saysthbot && \
    cargo build --release

FROM alpine

RUN apk add --no-cache proxychains4

ENV TGBOT_TOKEN="" DATABASE_URI="" WRAPPER="proxychain"

CMD ["-c", "${WRAPPER} ./saysthbot-reborn"]

ENTRYPOINT [ "/bin/sh" ]

COPY --from=build /saysthbot/target/release/saysthbot-reborn ./
