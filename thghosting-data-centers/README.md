# thghosting-data-centers

* [Cargo package](https://crates.io/crates/thghosting-data-centers)

## Dev

```
curl https://www.thghosting.com/network/data-centers/ -o tests/data-centers.html
```

```
sudo pacman -S js-beautify

curl -sS https://thg-hosting-master100tbcorp.netdna-ssl.com/views/js/production-stable.min.js \
    | js-beautify \
    | sed -n '/^            dataCentres: \[{$/,/^            }\]$/p' \
    | sed '1i{' \
    | sed '$a}' \
    | node -r fs -e 'console.log(JSON.stringify(eval(fs.readFileSync("/dev/stdin", "utf-8")), null, 4));' \
    > tests/data_centers.json
```
