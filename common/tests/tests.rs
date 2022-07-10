use common::*;

#[test]
fn should_serde_empty_data() {
    let otw = OTW::new(Cmd::GetStats, Data::Empty).unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);

    let otw = OTW::new(Cmd::SaveConfig, Data::Empty).unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);
}

#[test]
fn should_fail_with_invalid_pair() {
    let empty = Data::Empty;
    let config = Data::Config(Config {
        id: FanId::F1,
        min_temp: 0.0,
        max_temp: 1.0,
        min_duty: 20.0,
        max_duty: 23.0,
        enabled: false,
    });
    let stats = Data::Stats(Stats {
        rpm1: f32::MAX,
        rpm2: f32::MAX,
        rpm3: f32::MAX,
        rpm4: f32::MAX,
        temp1: f32::MAX,
    });
    let fan_id = Data::FanId(FanId::F1);
    let response = Data::Result(Response::Ok);

    OTW::new(Cmd::SaveConfig, empty.clone()).unwrap();
    OTW::new(Cmd::SaveConfig, config.clone()).unwrap_err();
    OTW::new(Cmd::SaveConfig, stats.clone()).unwrap_err();
    OTW::new(Cmd::SaveConfig, fan_id.clone()).unwrap_err();
    OTW::new(Cmd::SaveConfig, response.clone()).unwrap_err();

    OTW::new(Cmd::GetStats, empty.clone()).unwrap();
    OTW::new(Cmd::GetStats, config.clone()).unwrap_err();
    OTW::new(Cmd::GetStats, stats.clone()).unwrap_err();
    OTW::new(Cmd::GetStats, fan_id.clone()).unwrap_err();
    OTW::new(Cmd::GetStats, response.clone()).unwrap_err();

    OTW::new(Cmd::Config, empty.clone()).unwrap_err();
    OTW::new(Cmd::Config, config.clone()).unwrap();
    OTW::new(Cmd::Config, stats.clone()).unwrap_err();
    OTW::new(Cmd::Config, fan_id.clone()).unwrap_err();
    OTW::new(Cmd::Config, response.clone()).unwrap_err();

    OTW::new(Cmd::SetConfig, empty.clone()).unwrap_err();
    OTW::new(Cmd::SetConfig, config.clone()).unwrap();
    OTW::new(Cmd::SetConfig, stats.clone()).unwrap_err();
    OTW::new(Cmd::SetConfig, fan_id.clone()).unwrap_err();
    OTW::new(Cmd::SetConfig, response.clone()).unwrap_err();

    OTW::new(Cmd::Stats, empty.clone()).unwrap_err();
    OTW::new(Cmd::Stats, config.clone()).unwrap_err();
    OTW::new(Cmd::Stats, stats.clone()).unwrap();
    OTW::new(Cmd::Stats, fan_id.clone()).unwrap_err();
    OTW::new(Cmd::Stats, response.clone()).unwrap_err();

    OTW::new(Cmd::GetConfig, empty.clone()).unwrap_err();
    OTW::new(Cmd::GetConfig, config.clone()).unwrap_err();
    OTW::new(Cmd::GetConfig, stats.clone()).unwrap_err();
    OTW::new(Cmd::GetConfig, fan_id.clone()).unwrap();
    OTW::new(Cmd::GetConfig, response.clone()).unwrap_err();

    OTW::new(Cmd::Result, empty.clone()).unwrap_err();
    OTW::new(Cmd::Result, config.clone()).unwrap_err();
    OTW::new(Cmd::Result, stats.clone()).unwrap_err();
    OTW::new(Cmd::Result, fan_id.clone()).unwrap_err();
    OTW::new(Cmd::Result, response.clone()).unwrap();
    // consider a testing framework at this point
}

#[test]
fn should_serde_stats_data() {
    let otw = OTW::new(
        Cmd::Stats,
        Data::Stats(Stats {
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
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);

    let otw = OTW::new(
        Cmd::Stats,
        Data::Stats(Stats {
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
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);

    let otw = OTW::new(
        Cmd::Stats,
        Data::Stats(Stats {
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
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);
}

#[test]
fn should_serde_config_data() {
    let otw = OTW::new(
        Cmd::Config,
        Data::Config(Config {
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
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);

    let otw = OTW::new(
        Cmd::Config,
        Data::Config(Config {
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
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);
}

#[test]
fn should_serde_standby() {
    let otw = OTW::new(Cmd::SetStandby, Data::U64(200000)).unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);
}

#[test]
fn should_serde_configs() {
    let mut configs = Configs::default();
    println!("{:?}", configs);
    let vec = configs.to_vec().unwrap();
    println!("{}\n {:?}", vec.len(), vec);
    let res = Configs::from_bytes(&vec).unwrap();
    assert_eq!(res, configs);
    configs.data[0] = Config {
        id: FanId::F1,
        min_temp: 0.0,
        max_temp: 0.0,
        min_duty: 0.0,
        max_duty: 0.0,
        enabled: false,
    };
    println!("{:?}", configs);
    let vec = configs.to_vec().unwrap();
    println!("{}\n {:?}", vec.len(), vec);
    let res = Configs::from_bytes(&vec).unwrap();
    assert_eq!(res, configs);
}
