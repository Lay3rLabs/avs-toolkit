#!/bin/bash

set -e

# dynamically determine if we need sudo
SUDO=""
if ! groups | grep -q docker; then
  SUDO="sudo"
fi

U="cosmwasm"
V="0.16.0"

M=$(uname -m)
#M="x86_64" # Force Intel arch

A="linux/${M/x86_64/amd64}"
S=${M#x86_64}
S=${S:+-$S}

# We pass our ssh agent to the docker container, so it can pull dev dependencies properly
if [[ $OSTYPE == 'darwin'* ]]; then
  $SUDO docker run --platform $A --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  -v /run/host-services/ssh-auth.sock:/run/host-services/ssh-auth.sock -e SSH_AUTH_SOCK="/run/host-services/ssh-auth.sock" \
  $U/optimizer$S:$V
else
  # Ensure SSH agent is running and SSH_AUTH_SOCK is available
  if [ -z "$SSH_AUTH_SOCK" ]; then
    echo "SSH_AUTH_SOCK is not set. Starting ssh-agent..."
    eval $(ssh-agent)
    echo "SSH agent started with SSH_AUTH_SOCK=$SSH_AUTH_SOCK"
  fi

  $SUDO docker run --platform $A --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  --volume $SSH_AUTH_SOCK:/ssh-agent --env SSH_AUTH_SOCK=/ssh-agent \
  $U/optimizer$S:$V
fi
