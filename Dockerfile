ARG CROSS_BASE_IMAGE
FROM $CROSS_BASE_IMAGE

ARG CROSS_OS
ARG CROSS_TRIPLE

COPY ./kerberos.sh /kerberos.sh
RUN apt-get update && \
  apt-get install bison -y && \
  rm -rf /var/cache/apt
RUN bash /kerberos.sh $CROSS_OS $CROSS_TRIPLE
ENV RUSTFLAGS='-C relocation-model=static -lkrb5 -lk5crypto -lkrb5support -lcom_err -lc'
