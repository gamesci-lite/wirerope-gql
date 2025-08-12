FROM m.daocloud.io/docker.io/debian:bookworm-slim
WORKDIR /usr/local/bin
COPY target/release/hs_frog /usr/local/bin/hs_frog
CMD [ "./hs_frog" ]