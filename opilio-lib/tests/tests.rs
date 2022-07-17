use opilio_lib::*;

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
    let config = Data::Setting(FanSetting::new(Id::F1));
    let stats = Data::Stats(Stats {
        pump1_rpm: f32::MAX,
        fan1_rpm: f32::MAX,
        fan2_rpm: f32::MAX,
        fan3_rpm: f32::MAX,
        liquid_temp: f32::MAX,
        ambient_temp: f32::MAX,
    });
    let fan_id = Data::SettingId(Id::P1);
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

    OTW::new(Cmd::UploadSetting, empty.clone()).unwrap_err();
    OTW::new(Cmd::UploadSetting, config.clone()).unwrap();
    OTW::new(Cmd::UploadSetting, stats.clone()).unwrap_err();
    OTW::new(Cmd::UploadSetting, fan_id.clone()).unwrap_err();
    OTW::new(Cmd::UploadSetting, response.clone()).unwrap_err();

    OTW::new(Cmd::Stats, empty.clone()).unwrap_err();
    OTW::new(Cmd::Stats, config.clone()).unwrap_err();
    OTW::new(Cmd::Stats, stats.clone()).unwrap();
    OTW::new(Cmd::Stats, fan_id.clone()).unwrap_err();
    OTW::new(Cmd::Stats, response.clone()).unwrap_err();

    OTW::new(Cmd::GetConfig, empty.clone()).unwrap();
    OTW::new(Cmd::GetConfig, config.clone()).unwrap_err();
    OTW::new(Cmd::GetConfig, stats.clone()).unwrap_err();
    OTW::new(Cmd::GetConfig, fan_id.clone()).unwrap_err();
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
            pump1_rpm: 2.0,
            fan1_rpm: 0.0,
            fan2_rpm: 1.0,
            fan3_rpm: 20.0,
            liquid_temp: 23.0,
            ambient_temp: 20.0,
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
            pump1_rpm: 0.0,
            fan1_rpm: 0.0,
            fan2_rpm: 0.0,
            fan3_rpm: 0.0,
            liquid_temp: 0.1,
            ambient_temp: 0.0,
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
            pump1_rpm: f32::MAX,
            fan1_rpm: f32::MAX,
            fan2_rpm: f32::MAX,
            fan3_rpm: f32::MAX,
            liquid_temp: f32::MAX,
            ambient_temp: f32::MAX,
        }),
    )
    .unwrap();
    println!("{:?}", otw);
    let vec = otw.to_vec().unwrap();
    println!("{:?}", vec);
    let otwb = OTW::from_bytes(&vec).unwrap();
    assert_eq!(otw, otwb);
}

// #[test]
// fn should_serde_config_data() {
//     let otw = OTW::new(Cmd::Config, Data::Config(Config::default())).unwrap();
//     println!("{:?}", otw);
//     let vec = otw.to_vec().unwrap();
//     println!("{:?}", vec);
//     let otwb = OTW::from_bytes(&vec).unwrap();
//     assert_eq!(otw, otwb);

//     let otw = OTW::new(Cmd::Config, Data::Config(Config::default())).unwrap();
//     println!("{:?}", otw);
//     let vec = otw.to_vec().unwrap();
//     assert!(vec.len() <= MAX_SERIAL_DATA_SIZE);
//     println!("{:?}", vec);
//     let otwb = OTW::from_bytes(&vec).unwrap();
//     assert_eq!(otw, otwb);
// }

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
    let mut configs = Config::default();
    println!("{:#?}", configs);
    println!("{}", serde_json::to_string_pretty(&configs).unwrap());
    let vec = configs.to_vec().unwrap();
    println!("{}\n {:?}", vec.len(), vec);
    let res = Config::from_bytes(&vec).unwrap();
    assert_eq!(res, configs);
    // configs.data[0] = FanSetting {
    //     id: Id::P1,
    //     min_temp: 0.0,
    //     max_temp: 0.0,
    //     min_duty: 0.0,
    //     max_duty: 0.0,
    //     enabled: false,
    // };
    // println!("{:#?}", configs);
    let vec = configs.to_vec().unwrap();
    // println!("{}\n {:?}", vec.len(), vec);
    let res = Config::from_bytes(&vec).unwrap();
    assert_eq!(res, configs);
}

#[test]
fn should_calculate_duty() {
    let mut setting = FanSetting::new(Id::P1);
    // liner curve 0 to 100 inclusive
    setting.curve = [(0.0, 0.0), (25.0, 25.0), (50.0, 50.0), (100.0, 100.0)];
    println!("{:#?}", setting);
    println!("{}", serde_json::to_string_pretty(&setting).unwrap());
    let vec: heapless::Vec<u8, 256> = postcard::to_vec(&setting).unwrap();
    println!("{}", vec.len());

    let temps = (0u16..=100).into_iter().collect::<Vec<_>>();

    let duties = temps
        .iter()
        .map(|t| setting.get_duty(*t as f32, 100))
        .collect::<Vec<_>>();

    assert_eq!(temps, duties);

    let double_duties = temps
        .iter()
        .map(|t| setting.get_duty(*t as f32, 200))
        .collect::<Vec<_>>();

    assert_eq!(
        double_duties,
        duties.iter().map(|d| *d * 2).collect::<Vec<_>>()
    );
}
