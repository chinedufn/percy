# Used to host the isomorphic web app example

FROM rust:latest

WORKDIR /usr/src

COPY . .

RUN rustup update nightly
RUN rustup default nightly

# Node.js & npm
RUN curl -sL https://deb.nodesource.com/setup_9.x | bash -
RUN apt-get install -y nodejs

# Build tools for making npm install work
RUN apt-get install -y build-essential
RUN apt-get install -y libssl-dev
RUN apt-get install -y pkg-config

# Diagnostics
RUN pwd
RUN ls

RUN npm install

# TODO: Run an optimized production build instead.. maybe `./start-dev.sh` ./start-prod.sh
# or something..
RUN ./examples/isomorphic/start.sh
