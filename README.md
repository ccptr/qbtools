# Examples
## Export items to stdout in the JSON format
```sh
qbtools export items
```

## Export customers to stdout in the TOML format
```sh
qbtools export customers -f toml
```

## Export customers to a file
```sh
qbtools export customers --output-path customer-data.json
```

# Configuration
Run once to create an example JSON config file in your current directory.
The config file read in the following order:
- qb-api-cfg.json
- qb-api-cfg.toml
- qb-api-cfg.yaml
- qb-api-cfg.yml

# Warning
This crate is still in development, and things (such as the config file name/lookup order) may change at any time, and without warning, *especially* before the crate reaches version 0.1.0. However, I don't expect anything in the `Examples` section to break.
