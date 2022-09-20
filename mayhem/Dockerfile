# Build Stage
FROM ghcr.io/evanrichter/cargo-fuzz:latest as builder

## Add source code to the build stage.
ADD . /src
WORKDIR /src

RUN echo building instrumented harnesses && \
    bash -c "pushd fuzz && cargo +nightly -Z sparse-registry fuzz build deserialize_value && popd" && \
    bash -c "pushd fuzz && cargo +nightly -Z sparse-registry fuzz build read_region && popd" && \
    bash -c "pushd fuzz && cargo +nightly -Z sparse-registry fuzz build serialize_value && popd" && \
    mv fuzz/target/x86_64-unknown-linux-gnu/release/deserialize_value /deserialize_value && \
    mv fuzz/target/x86_64-unknown-linux-gnu/release/serialize_value /serialize_value && \
    mv fuzz/target/x86_64-unknown-linux-gnu/release/read_region /read_region && \
    echo done

RUN echo building non-instrumented harnesses && \
    export RUSTFLAGS="--cfg fuzzing -Clink-dead-code -Cdebug-assertions -C codegen-units=1" && \
    bash -c "pushd fuzz && cargo +nightly -Z sparse-registry build --release --bin deserialize_value && popd" && \
    bash -c "pushd fuzz && cargo +nightly -Z sparse-registry build --release --bin read_region && popd" && \
    bash -c "pushd fuzz && cargo +nightly -Z sparse-registry build --release --bin serialize_value && popd" && \
    mv fuzz/target/release/deserialize_value /deserialize_value_no_inst && \
    mv fuzz/target/release/serialize_value /serialize_value_no_inst && \
    mv fuzz/target/release/read_region /read_region_no_inst && \
    echo done

# Package Stage
FROM rustlang/rust:nightly

COPY --from=builder /deserialize_value /deserialize_value_no_inst /
COPY --from=builder /serialize_value /serialize_value_no_inst /
COPY --from=builder /read_region /read_region_no_inst /
