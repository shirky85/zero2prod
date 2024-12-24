

#[derive(serde::Deserialize, Clone)]
pub struct Properties {
    pub server_port: u16,
    pub server_host: String,
    pub author: String,
    pub specific_properties: SomeProperties,
    pub email_client: EmailClientProperties,
}

#[derive(serde::Deserialize, Clone)]
pub struct SomeProperties{
    pub first: String,
    pub second: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientProperties{
    pub base_url: String,
    pub sender: String,
}


pub fn get_configuration() -> Result<Properties, config::ConfigError> {

    // Initialise our configuration reader
    let properties = config::Config::builder()
    // Add configuration values from a file named `configuration.yaml`.
    .add_source(
        config::File::new("src/configuration.yaml", config::FileFormat::Yaml)
    )
    .build()?;
    // Try to convert the configuration values it read into
    // our Settings type
    properties.try_deserialize::<Properties>()
}