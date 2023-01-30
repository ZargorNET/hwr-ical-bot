FROM rustlang/rust:nightly
COPY . .
RUN cargo install --path .
RUN rm -r target/

CMD ["hwr-ical-bot"]
