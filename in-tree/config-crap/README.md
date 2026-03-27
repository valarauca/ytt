Crap Config
---

This repo holds a handful of questionable types
to make _advanced_ configuration writing easier.

These types include:

* `Boolean`: Because you need to support users putting "True"/"False" in strings.
* `NiceDuration`: A type that supported Go-Lang Style string based durations, e.g.: "2hr 45mins 5.2secs".
* `Regexp`: Because you need a way to serialize/deserialize regex patterns (as well as fully configure them).
* `WithEnv<T>`: Because need to support shell-like substitution from your environment automatically.
* `WithFile<D,T>`: Because including a full configuration locally would bloat the master config, just just write a path to that other file.

# Features

* `default`: Pulls in everything, you probably do not want this.
* `ron`: Will create middleware types to support RON.
* `json5`: Will create middleware types to support JSON5.
* `toml`: Will create middleware types to support TOML.
* `json`: Will create middleware types to support JSON.
* `yaml`: Will create middleware types to support YAML.
* `minijinja`: Enables templating. 

# Why?

1. Hardcoded constants are bad.
2. Updating configs is easier then writing code.
3. When you modify your config at 11pm on friday night, you don't want the fact your wrapped a number in quotes to cause a fix to fail to deploy.
