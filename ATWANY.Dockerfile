FROM rust:1.31


LABEL VERSION="0.1.0"
LABEL AUTHOR="AhmedKorim <ahmedkorrim@gmail.com>"

WORKDIR /usr/src/atwany

COPY . .
RUN cargo install --path .

VOLUME /images
VOLUME /files

ENV RUST_BACKTRACE=full
EXPOSE 50051
CMD ["atwany"]
