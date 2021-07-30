# DocSearch

[![Build Status][build-img]][build-url]
[![Repository][crates-img]][crates-url]
[![Documentation][doc-img]][doc-url]

[build-img]: https://img.shields.io/github/workflow/status/dnaka91/docsearch/CI/main?style=for-the-badge
[build-url]: https://github.com/dnaka91/docsearch/actions?query=workflow%3ACI
[crates-img]: https://img.shields.io/crates/v/docsearch?style=for-the-badge
[crates-url]: https://crates.io/crates/docsearch
[doc-img]: https://img.shields.io/badge/docs.rs-docsearch-4d76ae?style=for-the-badge
[doc-url]: https://docs.rs/docsearch

Use the latest search index from `rustdoc` to find the docs.rs (or stdlib) URL for any item in a
crate by its [simle path](https://doc.rust-lang.org/stable/reference/paths.html#simple-paths).

## Usage

Add `docsearch` to your project with `cargo add docsearch` (needs [cargo-edit]) or add it manually
to your `Cargo.toml`:

```toml
[dependencies]
docsearch = "0.1.1"
```

In addition, you will need to use the lastest [tokio](https://tokio.rs) runtime to use this library
as it uses async/await and is bound to this runtime.

[cargo-edit]: https://github.com/killercup/cargo-edit

### Example

For examples check out the [search](examples/search.rs) example or consult the [docs](doc-url).

## License

This project is licensed under [MIT License](LICENSE) (or <http://opensource.org/licenses/MIT>).
