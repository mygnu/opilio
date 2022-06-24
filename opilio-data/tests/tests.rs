use opilio_data::*;

#[test]
fn config_test() {
    let config = Config::new(FanId::F1);

    let data = config.to_vec().unwrap();

    println!("{:?}, len: {}", data, data.len());
    println!("{:?}", CONFIG_SIZE);
    println!("{:?}", config);

    let result = Config::from_bytes(data.as_ref()).unwrap();

    assert_eq!(config, result);
}
