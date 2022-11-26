use opilio_lib::*;

#[test]
fn should_fail_with_invalid_pair() {
    let fan = FanSetting::new(Id::F2);
    let default_config = Config::default();
    let empty = DataRef::Empty;
    let config = DataRef::Config(&default_config);
    let setting = DataRef::Setting(&fan);
    let stats = DataRef::Stats(&Stats {
        pump1_rpm: f32::MAX,
        fan1_rpm: f32::MAX,
        fan2_rpm: f32::MAX,
        fan3_rpm: f32::MAX,
        liquid_temp: f32::MAX,
        liquid_out_temp: f32::MAX,
        ambient_temp: f32::MAX,
    });
    let id = DataRef::SettingId(&Id::P1);
    let response = DataRef::Result(&Response::Ok);

    OTW::serialised_vec(Cmd::SaveConfig, empty.clone()).unwrap();
    OTW::serialised_vec(Cmd::SaveConfig, config.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::SaveConfig, stats.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::SaveConfig, id.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::SaveConfig, response.clone()).unwrap_err();

    OTW::serialised_vec(Cmd::GetStats, empty.clone()).unwrap();
    OTW::serialised_vec(Cmd::GetStats, config.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::GetStats, stats.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::GetStats, id.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::GetStats, response.clone()).unwrap_err();

    OTW::serialised_vec(Cmd::Config, empty.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Config, config.clone()).unwrap();
    OTW::serialised_vec(Cmd::Config, stats.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Config, id.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Config, response.clone()).unwrap_err();

    OTW::serialised_vec(Cmd::UploadSetting, empty.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::UploadSetting, setting.clone()).unwrap();
    OTW::serialised_vec(Cmd::UploadSetting, stats.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::UploadSetting, id.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::UploadSetting, response.clone()).unwrap_err();

    OTW::serialised_vec(Cmd::Stats, empty.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Stats, config.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Stats, stats.clone()).unwrap();
    OTW::serialised_vec(Cmd::Stats, id.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Stats, response.clone()).unwrap_err();

    OTW::serialised_vec(Cmd::GetConfig, empty.clone()).unwrap();
    OTW::serialised_vec(Cmd::GetConfig, config.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::GetConfig, stats.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::GetConfig, id.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::GetConfig, response.clone()).unwrap_err();

    OTW::serialised_vec(Cmd::Result, empty.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Result, config.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Result, stats.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Result, id.clone()).unwrap_err();
    OTW::serialised_vec(Cmd::Result, response.clone()).unwrap();
    // consider a testing framework at this point
}

#[test]
fn should_serde_stats_data() {
    let stats = Stats {
        pump1_rpm: 2.0,
        fan1_rpm: 0.0,
        fan2_rpm: 1.0,
        fan3_rpm: 20.0,
        liquid_temp: 23.0,
        liquid_out_temp: 23.0,
        ambient_temp: 20.0,
    };

    let vec = OTW::serialised_vec(Cmd::Stats, DataRef::Stats(&stats)).unwrap();
    println!("{:?}", vec);
    let otw = OTW::from_bytes(&vec).unwrap();
    assert_eq!(
        otw,
        OTW {
            cmd: Cmd::Stats,
            data: Data::Stats(stats)
        }
    );
}

#[test]
fn should_serde_configs() {
    let mut configs = Config::default();
    println!("{:#?}", configs);
    let vec = configs.to_vec().unwrap();
    println!("{}\n {:?}", vec.len(), vec);
    let res = Config::from_bytes(&vec).unwrap();
    assert_eq!(res, configs);

    let vec = configs.to_vec().unwrap();

    let res = Config::from_bytes(&vec).unwrap();
    assert_eq!(res, configs);

    configs.smart_mode = None;
    println!("{:#?}", configs);
    let vec = configs.to_vec().unwrap();
    println!("{}\n {:?}", vec.len(), vec);
    let res = Config::from_bytes(&vec).unwrap();
    assert_eq!(res, configs);

    let vec = configs.to_vec().unwrap();

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

#[test]
fn should_calculate_smart_duty() {
    let duty = get_smart_duty(40.0, 20.0, 5.0, 100.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 36);

    let duty = get_smart_duty(40.0, 20.0, 5.0, 40.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 100);

    let duty = get_smart_duty(35.0, 20.0, 5.0, 100.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 30);

    let duty = get_smart_duty(30.0, 20.0, 5.0, 100.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 25);

    let duty = get_smart_duty(30.0, 20.0, 5.0, 40.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 46);

    let duty = get_smart_duty(35.0, 20.0, 5.0, 40.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 73);

    let duty = get_smart_duty(40.0, 20.0, 5.0, 100.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 36);

    let duty = get_smart_duty(40.0, 20.0, 5.0, 40.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 100);

    let duty = get_smart_duty(25.0, 20.0, 5.0, 100.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 20);

    let duty = get_smart_duty(25.0, 20.0, 5.0, 100.0, 100, false);
    println!("duty: {duty}");
    assert_eq!(duty, 0);

    let duty = get_smart_duty(25.1, 20.0, 5.0, 100.0, 100, false);
    println!("duty: {duty}");
    assert_eq!(duty, 20);

    let duty = get_smart_duty(24.0, 20.0, 5.0, 100.0, 100, false);
    println!("duty: {duty}");
    assert_eq!(duty, 0);

    let duty = get_smart_duty(20.0, 20.0, 5.0, 100.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 0);

    let duty = get_smart_duty(22.0, 20.0, 5.0, 100.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 0);

    let duty = get_smart_duty(23.0, 20.0, 5.0, 100.0, 100, true);
    println!("duty: {duty}");
    assert_eq!(duty, 0);

    let duty = get_smart_duty(20.0, 20.0, 5.0, 100.0, 100, false);
    println!("duty: {duty}");
    assert_eq!(duty, 0);
}

#[test]
fn should_create_default_ok() {
    let bytes = OTW::serialised_ok();

    println!("{bytes:?}");

    let otw = OTW::from_bytes(bytes).unwrap();
    println!("{otw:?}");

    assert_eq!(
        otw,
        OTW {
            cmd: Cmd::Result,
            data: Data::Result(Response::Ok)
        }
    )
}
