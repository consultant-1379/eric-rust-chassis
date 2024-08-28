# syntax=docker/dockerfile:1
ARG CBOS_VER
FROM armdocker.rnd.ericsson.se/proj-ldc/common_base_os_release/sles:${CBOS_VER}

ARG CBOS_VER
ARG CBOS_REPO=arm.rnd.ki.sw.ericsson.se/artifactory/proj-ldc-repo-rpm-local/common_base_os/sles/
ARG CBOS_REPO_DEV=arm.rnd.ki.sw.ericsson.se/artifactory/proj-ldc-repo-rpm-local/adp-dev/adp-build-env/

RUN zypper addrepo --gpgcheck-strict --refresh https://${CBOS_REPO}${CBOS_VER} cbos-repo && \
    zypper addrepo --gpgcheck-strict --refresh https://${CBOS_REPO}${CBOS_VER}_devel cbos-api-header && \
    zypper addrepo --gpgcheck-strict --refresh https://${CBOS_REPO_DEV}${CBOS_VER} cbos-repo-devel && \
    zypper --gpg-auto-import-keys refresh && \
    zypper install --no-confirm --name \
        cargo1.77  \
        cmake-3.20.4 \
        gcc \
        gcc-c++ \
        rust1.77
