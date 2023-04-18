FROM rust:1.31

WORKDIR /usr/src/turtlers
COPY . .

RUN cargo install --path .

CMD ["turtlers"]
