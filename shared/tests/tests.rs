use shared::*;

#[test]
fn test_empty_data() {
    let otw = OverTheWire::new(Command::GetStats, OtwData::Empty).unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OverTheWire::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);

    let otw = OverTheWire::new(Command::SaveConfig, OtwData::Empty).unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OverTheWire::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);
}

#[test]
fn test_stats_data() {
    let otw = OverTheWire::new(
        Command::Stats,
        OtwData::Stats(Stats {
            rpm1: 2.0,
            rpm2: 0.0,
            rpm3: 1.0,
            rpm4: 20.0,
            temp1: 23.0,
        }),
    )
    .unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OverTheWire::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);

    let otw = OverTheWire::new(
        Command::Stats,
        OtwData::Stats(Stats {
            rpm1: 0.0,
            rpm2: 0.0,
            rpm3: 0.0,
            rpm4: 0.0,
            temp1: 0.1,
        }),
    )
    .unwrap();

    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OverTheWire::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);

    let otw = OverTheWire::new(
        Command::Stats,
        OtwData::Stats(Stats {
            rpm1: f32::MAX,
            rpm2: f32::MAX,
            rpm3: f32::MAX,
            rpm4: f32::MAX,
            temp1: f32::MAX,
        }),
    )
    .unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OverTheWire::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);
}

#[test]
fn test_config_data() {
    let otw = OverTheWire::new(
        Command::Config,
        OtwData::Config(Config {
            id: FanId::F1,
            min_temp: 0.0,
            max_temp: 1.0,
            min_duty: 20.0,
            max_duty: 23.0,
            enabled: false,
        }),
    )
    .unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OverTheWire::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);

    let otw = OverTheWire::new(
        Command::Config,
        OtwData::Config(Config {
            id: FanId::F1,
            min_temp: f32::MAX,
            max_temp: f32::MAX,
            min_duty: f32::MAX,
            max_duty: f32::MAX,
            enabled: false,
        }),
    )
    .unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    assert!(vec.len() <= MAX_SERIAL_DATA_SIZE);
    println!("{:?}", vec);
    let otwb = OverTheWire::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);
}
