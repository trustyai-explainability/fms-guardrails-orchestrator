ARG BASE_IMAGE=registry.access.redhat.com/ubi9/ubi-minimal:latest@sha256:5b74fce9d6e629942a0c6dc0f546c193e70d7f974d999a48c948c53dd3d36362
ARG CONFIG_FILE=config/config.yaml

## Rust builder ################################################################
FROM ${BASE_IMAGE} AS rust-builder

RUN microdnf install -y --disableplugin=subscription-manager \
        rust \
        cargo \
        gcc \
        openssl-devel && \
    microdnf clean all --disableplugin=subscription-manager

WORKDIR /app

## Orchestrator builder #########################################################
FROM rust-builder AS fms-guardrails-orchestr8-builder
ARG CONFIG_FILE=config/config.yaml

COPY build.rs *.toml Cargo.lock LICENSE /app/
COPY ${CONFIG_FILE} /app/config/config.yaml
COPY protos/ /app/protos/
COPY src/ /app/src/
COPY tests/ /app/tests/

RUN cargo build --release

COPY tests/resources /app/tests/resources
RUN cargo test

## Release image ################################################################
FROM ${BASE_IMAGE} AS fms-guardrails-orchestr8-release
ARG CONFIG_FILE=config/config.yaml

COPY --from=fms-guardrails-orchestr8-builder /app/target/release/fms-guardrails-orchestr8 /app/bin/
COPY ${CONFIG_FILE} /app/config/config.yaml

RUN microdnf install -y --disableplugin=subscription-manager shadow-utils && \
    microdnf clean all --disableplugin=subscription-manager

RUN groupadd --system orchestr8 --gid 1001 && \
    adduser --system --uid 1001 --gid 0 --groups orchestr8 \
    --create-home --home-dir /app --shell /sbin/nologin \
    --comment "FMS Orchestrator User" orchestr8

# STIG hardening
RUN --mount=type=bind,source=scripts,target=scripts \
    sh scripts/installRemediationTools.sh && \
    sh scripts/remediation-script.sh && \
    sh scripts/removeRemediationTools.sh

USER orchestr8

HEALTHCHECK NONE

ENV ORCHESTRATOR_CONFIG=/app/config/config.yaml

CMD ["/app/bin/fms-guardrails-orchestr8"]

LABEL com.redhat.component="odh-fms-guardrails-orchestrator-rhel9" \
      name="rhoai/odh-fms-guardrails-orchestrator-rhel9" \
      io.k8s.display-name="odh-fms-guardrails-orchestrator-rhel9" \
      io.k8s.description="REST API orchestrator coordinating AI text generation with safety guardrails" \
      description="REST API orchestrator coordinating AI text generation with safety guardrails" \
      summary="Guardrails Orchestrator for detector-based content safety filtering" \
      com.redhat.license_terms="https://www.redhat.com/licenses/Red_Hat_Standard_EULA_20191108.pdf"
