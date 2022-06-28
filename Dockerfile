FROM rust as build

WORKDIR /usr/src/saysthbot
COPY . .
RUN rustup default nightly && cargo build --release

FROM debian:stable-slim

RUN apt update && apt install -y proxychains4 ca-certificates && apt clean
ENV TGBOT_TOKEN="" DATABASE_URI="" WRAPPER=""
CMD ["-c", "${WRAPPER} ./saysthbot-reborn ${OPTIONS}"]
ENTRYPOINT [ "/bin/sh" ]

COPY --from=build /usr/src/saysthbot/target/release/saysthbot-reborn ./
