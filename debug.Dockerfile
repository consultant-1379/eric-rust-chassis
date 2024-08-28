# syntax=docker/dockerfile:1
ARG CBOS_VER
FROM armdocker.rnd.ericsson.se/proj-ldc/common_base_os_release/sles:${CBOS_VER}

ARG CBOS_VER
ARG CBOS_REPO=arm.rnd.ki.sw.ericsson.se/artifactory/proj-ldc-repo-rpm-local/common_base_os/sles/
ARG CBOS_REPO_DEV=arm.rnd.ki.sw.ericsson.se/artifactory/proj-ldc-repo-rpm-local/adp-dev/adp-build-env/

RUN zypper addrepo --gpgcheck-strict --refresh https://${CBOS_REPO}${CBOS_VER} cbos-repo && \
    zypper addrepo --gpgcheck-strict --refresh https://${CBOS_REPO_DEV}${CBOS_VER} cbos-repo-devel && \
    zypper --gpg-auto-import-keys refresh && \
    zypper install --no-confirm --name \
        curl \
        iputils \
        netcat-openbsd

ARG USER_ID=199655
RUN echo "$USER_ID:!::0:::::" >>/etc/shadow

ARG USER_NAME="rusty-dbg"
RUN echo "$USER_ID:x:$USER_ID:0:An Identity for $USER_NAME:/nonexistent:/bin/false" >>/etc/passwd

USER $USER_ID

CMD ["/bin/sleep", "infinity"]
