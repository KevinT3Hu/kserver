FROM archlinux

COPY ./target/release/kserver /usr/local/bin/kserver
CMD ["kserver"]