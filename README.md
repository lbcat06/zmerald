# zmerald
An opinionated configuration language ridden with flaws based on [RON](https://github.com/ron-rs/ron)

## motivation
As I started to work on some of my other projects such as [benten](https://github.com/liabri/benten) and [tema](https://github.com/liabri/tema) I used yaml for configuration and such, but it did not provide the flexibility I was looking for. I would like to eventually work on my own wayland compositor too and thought a good configuration language is essential. This all led to the birth of zmerald, a configuration language which allows various constructions to keep the config readable and easily maintainable. Details in the grammar.md.

## why the name `zmerald` ?
Thought emerald sounded cool, but it sounds even cooler in Maltese.

## todo
- include keyword;
- variables;
- allow comment on last line of file;
- rather than throwing an error on duplicate fields, add to sequence (if sequence) or override;
- remove need of semicolon `;` in constructions such as variables and cavetta;
- remove need of specifying field names in structs and base it on order of "arguments" (values);
- demonyms for cavetta construction, as in `key <id> value`, `key` is a demonym for hashmap `keys`. use field attributes for this ?