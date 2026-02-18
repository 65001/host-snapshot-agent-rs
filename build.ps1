# We use docker buildx to generate artifacts for multiple architectures, on windows, to ensure smooth builds
docker buildx build --build-arg TARGET=x86_64-unknown-linux-gnu --target export --output type=local,dest=$(pwd)/dist/x86_64-unknown-linux-gnu/ .
docker buildx build --build-arg TARGET=aarch64-unknown-linux-gnu --target export --output type=local,dest=$(pwd)/dist/aarch64-unknown-linux-gnu/ .
docker buildx build --build-arg TARGET=x86_64-unknown-linux-musl --target export --output type=local,dest=$(pwd)/dist/x86_64-unknown-linux-musl/ .
docker buildx build --build-arg TARGET=aarch64-unknown-linux-musl --target export --output type=local,dest=$(pwd)/dist/aarch64-unknown-linux-musl/ .