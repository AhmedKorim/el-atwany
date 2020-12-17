FROM alpine as alp
RUN apk add ca-certificates

FROM scratch
LABEL VERSION="0.1.0"
LABEL AUTHOR="AhmedKorim <ahmedkorrim@gmail.com>"


COPY --from=alp /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ADD build/waclient /
ENV RUST_BACKTRACE=full
EXPOSE 50051
CMD ["/waclient"]
