FROM --platform=$BUILDPLATFORM ghcr.io/drogue-iot/builder:0.1.19 as builder

RUN mkdir /build
ADD . /build
WORKDIR /build

RUN npm install
RUN trunk build --release

FROM registry.access.redhat.com/ubi8/nginx-120:latest

COPY --from=builder /build/dist/ ./

CMD [ "nginx", "-g", "daemon off;"]
