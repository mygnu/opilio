use shared::*;

#[test]
fn config_test() {
    let config = Config::new(FanId::F1);

    let data = config.to_vec().unwrap();

    println!("{:?}", config);
    println!("{:?}, len: {}", data, data.len());
    println!("{:?}", CONFIG_SIZE);

    let result = Config::from_bytes(data.as_ref()).unwrap();

    assert_eq!(config, result);
}

#[test]
fn command_test() {
    let serial_data = OverWireCmd::new(Command::GetConfig).data(
        heapless::Vec::from_slice(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            19, 20, 21, 22, 23, 24, 25, 26, 26, 28, 29, 30, 31,
        ])
        .unwrap(),
    );
    let data = serial_data.to_vec().unwrap();

    println!("{:?}", serial_data);
    println!("{:?}, len: {}", data, data.len());
    println!("{:?}", SERIAL_DATA_SIZE);

    let result = OverWireCmd::from_bytes(data.as_ref()).unwrap();

    assert_eq!(serial_data, result);
}

#[test]
fn command_rpm_data() {
    let rpm_data = RpmData {
        f1: 0.0,
        f2: 0.0,
        f3: 0.0,
        f4: 0.0,
    };

    let data = rpm_data.to_vec().unwrap();

    println!("{:?}", rpm_data);
    println!("{:?}, len: {}", data, data.len());
    println!("{:?}", RPM_DATA_SIZE);

    let result = RpmData::from_bytes(data.as_ref()).unwrap();

    assert_eq!(rpm_data, result);
}
