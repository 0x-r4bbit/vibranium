<a name="v0.1.0"></a>
## v0.1.0 Genesis (2019-06-11)


#### Breaking Changes

* **vibranium::config:**  introduce command to set config options ([0f00a6aa](https://github.com/pascalprecht/vibranium/commit/0f00a6aa62105e20fb542931bc8583f2ed9f3f16), closes [#12](https://github.com/pascalprecht/vibranium/issues/12), breaks [#](https://github.com/pascalprecht/vibranium/issues/))

#### Features

*   introduce deploy command ([a2b9c703](https://github.com/pascalprecht/vibranium/commit/a2b9c70385aab9bb203b908a6fd8fb31b7b54a0e), closes [#46](https://github.com/pascalprecht/vibranium/issues/46))
*   introduce blockchain section in project config ([9e2e4f1a](https://github.com/pascalprecht/vibranium/commit/9e2e4f1aa0d4b8a4b5d1b66e960ab3fc52c31c6a))
*   introduce --verbose flag for init, node and compile command ([8541d57f](https://github.com/pascalprecht/vibranium/commit/8541d57ffa47add0540abbd7ee4d544e505aac0f))
*   introduce --verbose flag in reset command ([5b9291be](https://github.com/pascalprecht/vibranium/commit/5b9291be7275361066cd2123d0618cbc594bc4ae))
*   improve compiler option configuration with vibranium.toml ([dba9b86c](https://github.com/pascalprecht/vibranium/commit/dba9b86cfdd626defecaf6148a20a769b40eedd2))
*   introduce config module for shared config related APIs ([7a5d1b64](https://github.com/pascalprecht/vibranium/commit/7a5d1b64846e5c234c5ee125821216135f40d1a9))
*   honor existing artifacts_dir when resetting vibranium project ([31fe7cf9](https://github.com/pascalprecht/vibranium/commit/31fe7cf9424d24248660137ac0d4c13b59e5ebaf))
*   introduce reset command ([1fba9cd1](https://github.com/pascalprecht/vibranium/commit/1fba9cd158d23cae8f5512c7c0a9f28c54941260))
*   generate default vibranium.toml file on init ([8255018e](https://github.com/pascalprecht/vibranium/commit/8255018e3582b6157c8e0e5ce60555acc8c7b86e))
*   introduce first super basic init command ([990e75d7](https://github.com/pascalprecht/vibranium/commit/990e75d7de6f2f4b125585f6d618d636610b1d84))
*   introduce first rudimentary `node` command ([94ff18ff](https://github.com/pascalprecht/vibranium/commit/94ff18ff5726677328847da46a67e2634190d350))
* **vibranium::blockchain:**
  *  introduce accounts commmand ([ea55d635](https://github.com/pascalprecht/vibranium/commit/ea55d635297b4ac80168bec87412428ed453f1e7), closes [#30](https://github.com/pascalprecht/vibranium/issues/30))
  *  add strategy algorithm to support different clients ([48c2583f](https://github.com/pascalprecht/vibranium/commit/48c2583f41f0a3c32d2ca7529137ffff569c238f), closes [#21](https://github.com/pascalprecht/vibranium/issues/21))
  *  enable client specific options using raw arguments ([de3edc27](https://github.com/pascalprecht/vibranium/commit/de3edc2765797bc1500f1358c1db62503327df6d))
  *  make node client configurable ([b2be8498](https://github.com/pascalprecht/vibranium/commit/b2be849836d7de48e0e539e13592593fca419a14))
* **vibranium::cli:**
  *  introduce --restore-config option in reset command ([dd763e25](https://github.com/pascalprecht/vibranium/commit/dd763e25479a3ca2dc250f3a775f0faa2bc0592f), closes [#15](https://github.com/pascalprecht/vibranium/issues/15))
  *  introduce --unset option in config command ([d23ff99f](https://github.com/pascalprecht/vibranium/commit/d23ff99f0579b76848f4b7f4b3e7b981f81e3db4))
* **vibranium::compiler:**
  *  introduce simple Solcjs strategy ([a7cc324f](https://github.com/pascalprecht/vibranium/commit/a7cc324f63bebea104c9507d99f0cd37698ded69))
  *  introduce default strategy to support any compiler ([1e55dbc1](https://github.com/pascalprecht/vibranium/commit/1e55dbc1cafa58f86bbb2f6c364c310c79d5506d))
  *  introduce rudimentary compilation support ([cd9c6428](https://github.com/pascalprecht/vibranium/commit/cd9c6428da3992443048849c77591f3af38c8cdd))
* **vibranium::config:**  introduce command to set config options ([0f00a6aa](https://github.com/pascalprecht/vibranium/commit/0f00a6aa62105e20fb542931bc8583f2ed9f3f16), closes [#12](https://github.com/pascalprecht/vibranium/issues/12), breaks [#](https://github.com/pascalprecht/vibranium/issues/))
* **vibranium::deployment:**  allow users to specify gas_price and gas_limit per Smart Contract ([ae9723b4](https://github.com/pascalprecht/vibranium/commit/ae9723b451b8fe352f28c164e6816fce1cefda6d))

#### Bug Fixes

*   through meaningful error message when trying to compile a non vibranium project ([2cb8ff5d](https://github.com/pascalprecht/vibranium/commit/2cb8ff5dde792eedb79624cae24db352d60a510f))
*   handle error correctly when Vibranium config is corrupted ([00e3d0db](https://github.com/pascalprecht/vibranium/commit/00e3d0db72eb635148456aa8e8cd027e847f688d))
* **ci:**  use powershell to put Ganache-CLI into the background ([db07ab96](https://github.com/pascalprecht/vibranium/commit/db07ab96780a8e8586062ee962fb59656ba778b5), closes [#49](https://github.com/pascalprecht/vibranium/issues/49))
* **vibranium::cli:**  filter out empty values when setting options ([fb4bf276](https://github.com/pascalprecht/vibranium/commit/fb4bf276ba4243ea60d96a6e2fe2be69a78f67a9))
* **vibranium::code_generator:**  ensure custom artifacts directory is being reset as well ([571796b8](https://github.com/pascalprecht/vibranium/commit/571796b85a99d1a7670b86afd39ecc35e719bda7))
* **vibranium::compiler:**
  *  determine the correct executable command on Windows ([28bb3004](https://github.com/pascalprecht/vibranium/commit/28bb30044597a316f72eea63b846fd7e2c18a807))
  *  treat stderr as error output ([27f686cb](https://github.com/pascalprecht/vibranium/commit/27f686cb92cf826f366cb21c8a824fe05ddf0113), closes [#2](https://github.com/pascalprecht/vibranium/issues/2))
  *  ensure proper error description is displayed ([d750f834](https://github.com/pascalprecht/vibranium/commit/d750f8349a2da5e0cec423a353d67ddd35598646))
  *  ensure glob pattern is composed correctly ([e61f6506](https://github.com/pascalprecht/vibranium/commit/e61f6506620a2da7a12c492d7c9c232dcaffb632))
* **vibranium::project_generator:**
  *  ensure correct glob pattern for contract sources ([5199b5e8](https://github.com/pascalprecht/vibranium/commit/5199b5e812b50a026d7bda30a6cb01da0c62236d))
  *  ensure vibranium folder preserves when reset fails ([d5090cc7](https://github.com/pascalprecht/vibranium/commit/d5090cc7bd26d10e313a2d1585920e23eb20d140))



