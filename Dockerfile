# syntax=docker/dockerfile:1
ARG CBOS_IMAGE_REPO
ARG CBOS_IMAGE_NAME
ARG CBOS_IMAGE_TAG
FROM ${CBOS_IMAGE_REPO}/${CBOS_IMAGE_NAME}:${CBOS_IMAGE_TAG}

COPY target/release/ves /opt/ericsson/bin/
COPY ms/res/schemas/* /opt/ericsson/share/

ARG USER_ID=200299
RUN echo "$USER_ID:!::0:::::" >>/etc/shadow

ARG USER_NAME="rusty"
RUN echo "$USER_ID:x:$USER_ID:0:An Identity for $USER_NAME:/nonexistent:/bin/false" >>/etc/passwd

USER $USER_ID

CMD ["/opt/ericsson/bin/ves"]

ARG COMMIT
ARG BUILD_DATE
ARG APP_VERSION
ARG RSTATE
ARG IMAGE_PRODUCT_NUMBER
LABEL \
    org.opencontainers.image.title=eric-rust-chassis \
    org.opencontainers.image.created=$BUILD_DATE \
    org.opencontainers.image.revision=$COMMIT \
    org.opencontainers.image.vendor=Ericsson \
    org.opencontainers.image.version=$APP_VERSION \
    com.ericsson.product-revision="${RSTATE}" \
    com.ericsson.product-number="$IMAGE_PRODUCT_NUMBER"
