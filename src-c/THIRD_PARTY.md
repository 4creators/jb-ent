# Third-Party Licenses

This project vendors third-party code. We are grateful to the authors and
maintainers of these projects for making their work freely available.

## Tree-sitter Runtime

The tree-sitter C runtime is vendored in `internal/cbm/vendored/ts_runtime/`.

- **Project:** [tree-sitter](https://github.com/tree-sitter/tree-sitter)
- **License:** MIT
- **Copyright:** (c) 2018 Max Brunsfeld

## Tree-sitter Grammars

Pre-generated parser sources are vendored in `internal/cbm/vendored/grammars/<lang>/`.
Each grammar is the work of its upstream authors; we vendor only the generated
`parser.c` (and `scanner.c` where applicable) for static compilation.
| Grammar    | Commit                                   | Upstream Repository                                                                           | License      | Copyright                                      |
|------------|------------------------------------------|-----------------------------------------------------------------------------------------------|--------------|------------------------------------------------|
| go         | 2346a3ab1bb3857b48b29d779a1ef9799a248cd7 | [tree-sitter/tree-sitter-go](https://github.com/tree-sitter/tree-sitter-go)                   | MIT          | (c) 2014 Max Brunsfeld                         |
| bash       | a06c2e4415e9bc0346c6b86d401879ffb44058f7 | [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)               | MIT          | (c) 2017 Max Brunsfeld                         |
| c          | ae19b676b13bdcc13b7665397e6d9b14975473dd | [tree-sitter/tree-sitter-c](https://github.com/tree-sitter/tree-sitter-c)                     | MIT          | (c) 2014 Max Brunsfeld                         |
| clojure    | e43eff80d17cf34852dcd92ca5e6986d23a7040f | [sogaiu/tree-sitter-clojure](https://github.com/sogaiu/tree-sitter-clojure)                   | EPL-1.0      | (c) Amaan Qureshi                              |
| cmake      | c7b2a71e7f8ecb167fad4c97227c838439280175 | [uyha/tree-sitter-cmake](https://github.com/uyha/tree-sitter-cmake)                           | MIT          | (c) Amaan Qureshi                              |
| cobol      | e99dbdc3d800d5fa2796476efd60af91f6b43d93 | [yutaro-sakamoto/tree-sitter-cobol](https://github.com/yutaro-sakamoto/tree-sitter-cobol)     | MIT          | (c) Amaan Qureshi                              |
| commonlisp | 32323509b3d9fe96607d151c2da2c9009eb13a2f | [theHamsta/tree-sitter-commonlisp](https://github.com/theHamsta/tree-sitter-commonlisp)       | MIT          | (c) Amaan Qureshi                              |
| cpp        | 8b5b49eb196bec7040441bee33b2c9a4838d6967 | [tree-sitter/tree-sitter-cpp](https://github.com/tree-sitter/tree-sitter-cpp)                 | MIT          | (c) 2014 Max Brunsfeld                         |
| c_sharp    | cac6d5fb595f5811a076336682d5d595ac1c9e85 | [tree-sitter/tree-sitter-c-sharp](https://github.com/tree-sitter/tree-sitter-c-sharp)         | MIT          | (c) 2014 Max Brunsfeld                         |
| css        | dda5cfc5722c429eaba1c910ca32c2c0c5bb1a3f | [tree-sitter/tree-sitter-css](https://github.com/tree-sitter/tree-sitter-css)                 | MIT          | (c) 2017 Max Brunsfeld                         |
| cuda       | 48b066f334f4cf2174e05a50218ce2ed98b6fd01 | [tree-sitter-grammars/tree-sitter-cuda](https://github.com/tree-sitter-grammars/tree-sitter-cuda) | MIT          | (c) Amaan Qureshi                              |
| dart       | 0fc19c3a57b1109802af41d2b8f60d8835c5da3a | [UserNobody14/tree-sitter-dart](https://github.com/UserNobody14/tree-sitter-dart)             | MIT          | (c) Amaan Qureshi                              |
| dockerfile | 971acdd908568b4531b0ba28a445bf0bb720aba5 | [camdencheek/tree-sitter-dockerfile](https://github.com/camdencheek/tree-sitter-dockerfile)   | MIT          | (c) Max Brunsfeld                              |
| elisp      | 29b4e49275f4a947ce17c8533bc20a1f97768c70 | [Wilfred/tree-sitter-elisp](https://github.com/Wilfred/tree-sitter-elisp)                     | MIT          | (c) Amaan Qureshi                              |
| elixir     | 7937d3b4d65fa574163cfa59394515d3c1cf16f4 | [elixir-lang/tree-sitter-elixir](https://github.com/elixir-lang/tree-sitter-elixir)           | MIT          | (c) Max Brunsfeld                              |
| elm        | 6d9511c28181db66daee4e883f811f6251220943 | [elm-tooling/tree-sitter-elm](https://github.com/elm-tooling/tree-sitter-elm)                 | MIT          | (c) Amaan Qureshi                              |
| erlang     | 520202ffbedfdb68048073af6040d102c9290dce | [WhatsApp/tree-sitter-erlang](https://github.com/WhatsApp/tree-sitter-erlang)                 | MIT          | (c) Nemo Anzai                                 |
| form       | SKIP                                     | Custom grammar                                                                                | MIT          | (c) 2026 DeusData                              |
| fortran    | 2597b9a3f551b7c1e4668ca14bbe1502cb8414dc | [stadelmanma/tree-sitter-fortran](https://github.com/stadelmanma/tree-sitter-fortran)         | MIT          | (c) Amaan Qureshi                              |
| fsharp     | 5247c1197cb290fcaea0e0a793d32829c1396831 | [ionide/tree-sitter-fsharp](https://github.com/ionide/tree-sitter-fsharp)                     | MIT          | (c) Amaan Qureshi                              |
| glsl       | 24a6c8ef698e4480fecf8340d771fbcb5de8fbb4 | [tree-sitter-grammars/tree-sitter-glsl](https://github.com/tree-sitter-grammars/tree-sitter-glsl) | MIT          | (c) Amaan Qureshi                              |
| graphql    | 5e66e961eee421786bdda8495ed1db045e06b5fe | [bkegley/tree-sitter-graphql](https://github.com/bkegley/tree-sitter-graphql)                 | MIT          | (c) Amaan Qureshi                              |
| groovy     | deb0dcf8c4544f07564060f6e9b9f6e4b0bfc27d | [murtaza64/tree-sitter-groovy](https://github.com/murtaza64/tree-sitter-groovy)               | MIT          | (c) Amaan Qureshi                              |
| haskell    | 0975ef72fc3c47b530309ca93937d7d143523628 | [tree-sitter/tree-sitter-haskell](https://github.com/tree-sitter/tree-sitter-haskell)         | MIT          | (c) 2017 Max Brunsfeld                         |
| hcl        | 64ad62785d442eb4d45df3a1764962dafd5bc98b | [tree-sitter-grammars/tree-sitter-hcl](https://github.com/tree-sitter-grammars/tree-sitter-hcl) | MIT          | (c) Amaan Qureshi                              |
| html       | 73a3947324f6efddf9e17c0ea58d454843590cc0 | [tree-sitter/tree-sitter-html](https://github.com/tree-sitter/tree-sitter-html)               | MIT          | (c) 2017 Max Brunsfeld                         |
| ini        | e4018b5176132b4f3c5d6e61cea383f42288d0f5 | [justinmk/tree-sitter-ini](https://github.com/justinmk/tree-sitter-ini)                       | MIT          | (c) Amaan Qureshi                              |
| java       | e10607b45ff745f5f876bfa3e94fbcc6b44bdc11 | [tree-sitter/tree-sitter-java](https://github.com/tree-sitter/tree-sitter-java)               | MIT          | (c) 2017 Max Brunsfeld                         |
| javascript | 58404d8cf191d69f2674a8fd507bd5776f46cb11 | [tree-sitter/tree-sitter-javascript](https://github.com/tree-sitter/tree-sitter-javascript)   | MIT          | (c) 2014 Max Brunsfeld                         |
| json       | 001c28d7a29832b06b0e831ec77845553c89b56d | [tree-sitter/tree-sitter-json](https://github.com/tree-sitter/tree-sitter-json)               | MIT          | (c) Max Brunsfeld                              |
| julia      | e0f9dcd180fdcfcfa8d79a3531e11d99e79321d3 | [tree-sitter/tree-sitter-julia](https://github.com/tree-sitter/tree-sitter-julia)             | MIT          | (c) Amaan Qureshi                              |
| kotlin     | 55622a49bd59ca42cec5c01ba5251bb4da9b8930 | [fwcd/tree-sitter-kotlin](https://github.com/fwcd/tree-sitter-kotlin)                         | MIT          | (c) Max Brunsfeld                              |
| lean       | efe6b87145608d12f5996bd7f0cf6095a0e82261 | [Julian/tree-sitter-lean](https://github.com/Julian/tree-sitter-lean)                         | MIT          | (c) Julian Samarrasinghe                       |
| lua        | 10fe0054734eec83049514ea2e718b2a56acd0c9 | [tree-sitter-grammars/tree-sitter-lua](https://github.com/tree-sitter-grammars/tree-sitter-lua) | MIT          | (c) Amaan Qureshi                              |
| magma      | SKIP                                     | Custom grammar                                                                                | MIT          | (c) 2026 DeusData                              |
| make       | 70613f3d812cbabbd7f38d104d60a409c4008b43 | [tree-sitter-grammars/tree-sitter-make](https://github.com/tree-sitter-grammars/tree-sitter-make) | MIT          | (c) Amaan Qureshi                              |
| markdown   | f969cd3ae3f9fbd4e43205431d0ae286014c05b5 | [tree-sitter-grammars/tree-sitter-markdown](https://github.com/tree-sitter-grammars/tree-sitter-markdown) | MIT          | (c) Amaan Qureshi                              |
| matlab     | c2390a59016f74e7d5f75ef09510768b4f30217e | [acristoffers/tree-sitter-matlab](https://github.com/acristoffers/tree-sitter-matlab)         | MIT          | (c) Alan Cristoffers                           |
| meson      | c84f3540624b81fc44067030afce2ff78d6ede05 | [Decodetalkers/tree-sitter-meson](https://github.com/Decodetalkers/tree-sitter-meson)         | MIT          | (c) Amaan Qureshi                              |
| nix        | eabf96807ea4ab6d6c7f09b671a88cd483542840 | [nix-community/tree-sitter-nix](https://github.com/nix-community/tree-sitter-nix)             | MIT          | (c) Amaan Qureshi                              |
| objc       | 181a81b8f23a2d593e7ab4259981f50122909fda | [tree-sitter-grammars/tree-sitter-objc](https://github.com/tree-sitter-grammars/tree-sitter-objc) | MIT          | (c) Max Brunsfeld                              |
| ocaml      | 2c40c1346a712cb517eda3195224ff4a95c8cc10 | [tree-sitter/tree-sitter-ocaml](https://github.com/tree-sitter/tree-sitter-ocaml)             | MIT          | (c) Max Brunsfeld                              |
| perl       | SKIP                                     | [tree-sitter-perl/tree-sitter-perl](https://github.com/tree-sitter-perl/tree-sitter-perl)     | MIT          | (c) Ganesh Tiwari (Requires local generation)  |
| php        | 3f2465c217d0a966d41e584b42d75522f2a3149e | [tree-sitter/tree-sitter-php](https://github.com/tree-sitter/tree-sitter-php)                 | MIT          | (c) Josh Vera, Max Brunsfeld, Amaan Qureshi    |
| protobuf   | e9f6b43f6844bd2189b50a422d4e2094313f6aa3 | [treywood/tree-sitter-proto](https://github.com/treywood/tree-sitter-proto)                   | MIT          | (c) Amaan Qureshi                              |
| python     | 26855eabccb19c6abf499fbc5b8dc7cc9ab8bc64 | [tree-sitter/tree-sitter-python](https://github.com/tree-sitter/tree-sitter-python)           | MIT          | (c) 2016 Max Brunsfeld                         |
| r          | 0e6ef7741712c09dc3ee6e81c42e919820cc65ef | [r-lib/tree-sitter-r](https://github.com/r-lib/tree-sitter-r)                                 | MIT          | (c) Max Brunsfeld                              |
| ruby       | ad907a69da0c8a4f7a943a7fe012712208da6dee | [tree-sitter/tree-sitter-ruby](https://github.com/tree-sitter/tree-sitter-ruby)               | MIT          | (c) 2017 Max Brunsfeld                         |
| rust       | 77a3747266f4d621d0757825e6b11edcbf991ca5 | [tree-sitter/tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)               | MIT          | (c) 2017 Max Brunsfeld                         |
| scala      | 0b3a557e666dcdcfae9aed991d93ddd6a902e9eb | [tree-sitter/tree-sitter-scala](https://github.com/tree-sitter/tree-sitter-scala)             | MIT          | (c) Amaan Qureshi                              |
| scss       | c478c6868648eff49eb04a4df90d703dc45b312a | [serenadeai/tree-sitter-scss](https://github.com/serenadeai/tree-sitter-scss)                 | MIT          | (c) Max Brunsfeld                              |
| sql        | c8f50f7660e2e0b72e51eaae68c8f7f9f9a01755 | [tjdevries/tree-sitter-sql](https://github.com/tjdevries/tree-sitter-sql)                     | MIT          | (c) Amaan Qureshi                              |
| svelte     | ae5199db47757f785e43a14b332118a5474de1a2 | [tree-sitter-grammars/tree-sitter-svelte](https://github.com/tree-sitter-grammars/tree-sitter-svelte) | MIT          | (c) Amaan Qureshi                              |
| swift      | db675450dcc1478ee128c96ecc61c13272431aab | [tree-sitter/tree-sitter-swift](https://github.com/tree-sitter/tree-sitter-swift)             | MIT          | (c) Max Brunsfeld                              |
| toml       | 64b56832c2cffe41758f28e05c756a3a98d16f41 | [tree-sitter-grammars/tree-sitter-toml](https://github.com/tree-sitter-grammars/tree-sitter-toml) | MIT          | (c) Amaan Qureshi                              |
| typescript | 75b3874edb2dc714fb1fd77a32013d0f8699989f | [tree-sitter/tree-sitter-typescript](https://github.com/tree-sitter/tree-sitter-typescript)   | MIT          | (c) Max Brunsfeld                              |
| tsx        | 75b3874edb2dc714fb1fd77a32013d0f8699989f | [tree-sitter/tree-sitter-typescript](https://github.com/tree-sitter/tree-sitter-typescript)   | MIT          | (c) Max Brunsfeld                              |
| verilog    | 227d277b6a1a5e2bf818d6206935722a7503de08 | [tree-sitter/tree-sitter-verilog](https://github.com/tree-sitter/tree-sitter-verilog)         | MIT          | (c) Aman Hardikar                              |
| vim        | 3092fcd99eb87bbd0fc434aa03650ba58bd5b43b | [tree-sitter-grammars/tree-sitter-vim](https://github.com/tree-sitter-grammars/tree-sitter-vim) | MIT          | (c) Amaan Qureshi                              |
| vue        | ce8011a414fdf8091f4e4071752efc376f4afb08 | [tree-sitter-grammars/tree-sitter-vue](https://github.com/tree-sitter-grammars/tree-sitter-vue) | MIT          | (c) Amaan Qureshi                              |
| wolfram    | ab3506a5b49b7d76a8ed06958d0b2b7be91a5d34 | [LumaKernel/tree-sitter-wolfram](https://github.com/LumaKernel/tree-sitter-wolfram)           | MIT          | (c) LumaKernel                                 |
| xml        | 5000ae8f22d11fbe93939b05c1e37cf21117162d | [tree-sitter-grammars/tree-sitter-xml](https://github.com/tree-sitter-grammars/tree-sitter-xml) | MIT          | (c) Amaan Qureshi                              |
| yaml       | 4463985dfccc640f3d6991e3396a2047610cf5f8 | [tree-sitter-grammars/tree-sitter-yaml](https://github.com/tree-sitter-grammars/tree-sitter-yaml) | MIT          | (c) Max Brunsfeld                              |
| zig        | 6479aa13f32f701c383083d8b28360ebd682fb7d | [tree-sitter-grammars/tree-sitter-zig](https://github.com/tree-sitter-grammars/tree-sitter-zig) | MIT          | (c) Max Brunsfeld                              |
| Library | License | Project |
| SQLite 3 | Public Domain | [sqlite.org](https://www.sqlite.org/) |
| mimalloc | MIT | [microsoft/mimalloc](https://github.com/microsoft/mimalloc) |
| Mongoose | Dual GPLv2 / Commercial | [cesanta/mongoose](https://github.com/cesanta/mongoose) |
| yyjson | MIT | [ibireme/yyjson](https://github.com/ibireme/yyjson) |
| xxHash | BSD-2-Clause | [Cyan4973/xxHash](https://github.com/Cyan4973/xxHash) |
| TRE | BSD-2-Clause | [laurikari/tre](https://github.com/laurikari/tre) |

## Notes

- **FORM** and **Magma** grammars are custom tree-sitter grammars created by DeusData
  for this project (MIT-licensed under the project's own license).
- **Clojure** uses the Eclipse Public License 1.0, which is compatible with MIT
  for downstream use.
- All other grammars are MIT-licensed.
