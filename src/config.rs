use std::{io, path::Path};

use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::*;
use quickbooks_ureq::{AccessToken, Quickbooks};

use core::result::Result;

// TODO: change these to production values once authorization flow is implemented
pub const CLIENT_ID: &str = "ABZYrDWkeAGWlwcMlNm9vvHjC0nTT4wXrwfjnuujJgBCr0sRJR";
pub const CLIENT_SECRET: &str = "TbFFiO2pdnHXnvAZkveZ8GXMQMxeHbhpggUDNmGe";

mod dummy {
    pub const COMPANY_ID: &str = "0000000000000000000";
    pub const ACCESS_TOKEN: &str = "XX00000000000Xxx0XXXXX0xxxxXXX0X0XXXxXxXxXXXxXxXxx";
    pub const REFRESH_TOKEN: &str = "XX00000000000Xxx0XXXXX0xxxxXXX0X0XXXxXxXxXXXxXxXxx";
    pub const TOKEN_TYPE: &str = "bearer";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub company_id: String,
    #[serde(flatten)]
    pub token: Option<AccessToken>,
}

impl From<QuickbooksConfig> for Config {
    fn from(config: QuickbooksConfig) -> Self {
        Self {
            company_id: config.company_id,
            token: Some(config.token),
        }
    }
}

impl Config {
    fn get_dummy_config() -> Self {
        let token = AccessToken {
            access_token: dummy::ACCESS_TOKEN.to_string(),
            refresh_token: dummy::REFRESH_TOKEN.to_string(),

            token_type: dummy::TOKEN_TYPE.to_string(),
        };

        Config {
            company_id: dummy::COMPANY_ID.to_string(),
            token: Some(token),
        }
    }

    pub fn get_example_json() -> Result<String, serde_json::Error> {
        let example = Self::get_dummy_config();
        serde_json::to_string_pretty(&example)
    }

    /// read config file, or write example config and exit the program
    pub fn read_or_write_and_exit(base_path: &str) -> Self {
        /// this should always exit
        fn fail(config: Result<Config, fs::Error>, base_path: &str) -> ! {
            let error = config.expect_err("programming error");

            log::error!(
                "failed to read {}, are you sure it has all necessary values? {error}",
                fs::get_possible_files(base_path),
            );

            let example_json = Config::get_example_json().unwrap_or_else(|err| {
                log::error!("failed to create an example JSON config file: {err}");
                exit(-2);
            });

            eprintln!("EXAMPLE CONFIG ({}.json):", base_path);
            eprintln!("{}", example_json);
            exit(2);
        }

        let config: Result<Self, fs::Error> = fs::read_config(Path::new(base_path));

        match config {
            Ok(ret) => ret,
            Err(ref err) => {
                match err {
                    fs::Error::IO(err) => {
                        if err.kind() == std::io::ErrorKind::NotFound {
                            log::error!("config file not found");

                            let example_config = Self::get_dummy_config();
                            example_config
                                .write_to(Path::new(base_path))
                                .expect("failed to write config");

                            log::info!(
                                "example config written to: {}",
                                fs::get_possible_files(base_path)
                            );
                            exit(1);
                        } else {
                            // this should always panic (and never return)
                            fail(config, base_path);
                        }
                    }
                    fs::Error::Deserialize(_msg) => fail(config, base_path),
                }
            }
        }
    }

    pub fn write_to(&self, base_path: &Path) -> Result<(), io::Error> {
        let file = fs::get_first_file(base_path);
        let file_type = fs::get_extension(&file);

        let out = match file_type.as_str() {
            "json" => {
                serde_json::to_string_pretty(self).expect("failed to serialize config to JSON")
            }
            #[cfg(feature = "toml")]
            "toml" => toml::to_string_pretty(self).expect("failed to serialize config to TOML"),
            #[cfg(feature = "yaml")]
            "yaml" | "yml" => {
                serde_yaml::to_string(self).expect("failed to serialize config to YAML")
            }
            _ => {
                panic!("this should not have happened");
            }
        };

        std::fs::write(file, out)?;

        return Ok(());
    }
}

pub fn get_authorized_qb(quiet: bool) -> Result<Quickbooks, commands::OutputError> {
    use quickbooks_types::CompanyInfo;

    fn make_client(config: &Config) -> Quickbooks {
        let token = config.token.clone().unwrap_or_else(|| {
            log::error!("QuickBooks OAuth workflow is not implemented; `token` is currently required in the config.");
            eprintln!("EXAMPLE CONFIG (.json):");
            eprintln!("{}", Config::get_example_json().unwrap_or_else(|err| {
                log::error!("failed to create example JSON config: {err}");
                exit(-1);
            }));
            exit(1);
        });

        let cfg = QuickbooksConfig {
            client_id: config::CLIENT_ID.to_string(),
            client_secret: config::CLIENT_SECRET.to_string(),

            base_url: QB_BASE_URL.to_string(),
            company_id: config.company_id.clone(),
            token,
            api: None,
        };

        Quickbooks::from(cfg)
    }

    fn print_company_info(company_info: &CompanyInfo) {
        log::info!("COMPANY INFO:");
        log::info!("Company name: {}", company_info.company_name);
        log::info!("Legal name:   {}", company_info.legal_name);
        log::info!("Company address: {}", company_info.company_addr);
        log::info!("Legal address:   {}", company_info.legal_addr);
    }

    let mut config = Config::read_or_write_and_exit(BASE_CONFIG_PATH);

    let has_refresh = if let Some(token) = config.token.clone() {
        !token.refresh_token.is_empty()
    } else {
        false
    };

    // Initialize the QuickBooks client.
    let mut qb = make_client(&config);

    if !has_refresh {
        log::error!("FATAL: no refresh token found");

        eprintln!("EXAMPLE CONFIG (.yml):");
        eprintln!(
            "{}",
            Config::get_example_json().unwrap_or_else(|err| {
                log::error!("failed to create example JSON config: {err}");
                exit(2);
            })
        );

        eprintln!();

        unimplemented!("need to set up an HTTPS redirect URI handler");

        // consent URL needed to retrieve authorization code, which needs to be exchanged for access and refresh tokens
        //let consent_url = qb.user_consent_url();

        //eprintln!("######################################################################");
        //eprintln!("###                          CONSENT URL                           ###");
        //eprintln!("###                                                                ###");
        //eprintln!("{}", consent_url);
        //eprintln!("###                                                                ###");
        //eprintln!("###   You must open the URL above to authorize this application.   ###");
        //eprintln!("###                                                                ###");
        //eprintln!("### Once completed, paste the contents of the web page into here:  ###");
        //let mut resp = String::new();
        //let stdin = std::io::stdin();
        //let auth_code: serde_json::Value = loop {
        //    stdin.read_line(&mut resp).expect("failed to read stdin");
        //    if let Ok(ret) = serde_json::from_str(&resp) {
        //        break ret;
        //    }
        //};
        //eprintln!("###                                                                ###");
        //eprintln!("######################################################################");
    }

    match qb.company_info() {
        Ok(response) => {
            let response: quickbooks_types::Response = match response.into_json() {
                Ok(response) => response,
                Err(err) => {
                    log::error!(
                        "failed to serialize response into a quickbooks_types::Response: {}",
                        err
                    );
                    exit(1);
                }
            };
            let company_info = &response.query_response.company_info[0];
            if !quiet {
                print_company_info(company_info);
            }
        }
        Err(error) => {
            use quickbooks_ureq::Error;

            match error {
                Error::Status(status_code, _response) => {
                    if status_code == StatusCode::UNAUTHORIZED {
                        match qb.refresh_access_token_with_reqwest() {
                            Ok(token) => {
                                if token != config.token.clone().unwrap() {
                                    log::trace!("AccessToken changed, writing to config...");
                                    config.token = Some(token);
                                    if let Err(error) = config.write_to(Path::new(BASE_CONFIG_PATH))
                                    {
                                        log::error!("failed to write config: {}", error);
                                        exit(1);
                                    }
                                }
                            }
                            Err(err) => {
                                log::error!("failed to refresh access token: {}", err);
                                eprintln!(
                                    "Try generating a new refresh token and placing it in {}",
                                    fs::get_possible_files(BASE_CONFIG_PATH)
                                );
                                exit(1);
                            }
                        }
                    } else {
                        let status_code = StatusCode::from_u16(status_code)
                            .expect("ureq not to report an invalid status code");
                        log::error!(
                            "Status code: {status_code} ({})",
                            status_code.canonical_reason().unwrap_or("unknown")
                        );
                    }
                }
                Error::Transport(transport) => {
                    log::error!(
                        "Transport error occured while attempting to retrieve company info: {transport}"
                    );
                    exit(3);
                }
            }

            let response: quickbooks_types::Response = qb
                .company_info()
                .expect("this should not have happened")
                .into_json()
                .expect("this should not have happened");

            if !quiet {
                let company_info = &response.query_response.company_info[0];
                print_company_info(company_info);
            }
        }
    }

    Ok(make_client(&config))
}
