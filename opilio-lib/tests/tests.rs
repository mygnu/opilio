use opilio_lib::*;

#[test]
fn should_fail_with_invalid_pair() {
    let default_config = Config::default();
    let empty = DataRef::Empty;
    let config = DataRef::Config(&default_config);

    let stats = DataRef::Stats(&Stats {
        pump1_rpm: f32::MAX,
        fan1_rpm: f32::MAX,
        fan2_rpm: f32::MAX,
        fan3_rpm: f32::MAX,
        liquid_temp: f32::MAX,
        liquid_out_temp: f32::MAX,
        ambient_temp: f32::MAX,
    });

    let response = DataRef::Result(&Response::Ok);

    OTW::serialised_vec(Msg::SaveConfig, empty.clone()).unwrap();
    OTW::serialised_vec(Msg::SaveConfig, config.clone()).unwrap_err();
    OTW::serialised_vec(Msg::SaveConfig, stats.clone()).unwrap_err();
    OTW::serialised_vec(Msg::SaveConfig, response.clone()).unwrap_err();

    OTW::serialised_vec(Msg::GetStats, empty.clone()).unwrap();
    OTW::serialised_vec(Msg::GetStats, config.clone()).unwrap_err();
    OTW::serialised_vec(Msg::GetStats, stats.clone()).unwrap_err();
    OTW::serialised_vec(Msg::GetStats, response.clone()).unwrap_err();

    OTW::serialised_vec(Msg::Config, empty.clone()).unwrap_err();
    OTW::serialised_vec(Msg::Config, config.clone()).unwrap();
    OTW::serialised_vec(Msg::Config, stats.clone()).unwrap_err();
    OTW::serialised_vec(Msg::Config, response.clone()).unwrap_err();

    OTW::serialised_vec(Msg::Ping, empty.clone()).unwrap();
    OTW::serialised_vec(Msg::Ping, stats.clone()).unwrap_err();
    OTW::serialised_vec(Msg::Ping, response.clone()).unwrap_err();

    OTW::serialised_vec(Msg::Stats, empty.clone()).unwrap_err();
    OTW::serialised_vec(Msg::Stats, config.clone()).unwrap_err();
    OTW::serialised_vec(Msg::Stats, stats.clone()).unwrap();
    OTW::serialised_vec(Msg::Stats, response.clone()).unwrap_err();

    OTW::serialised_vec(Msg::GetConfig, empty.clone()).unwrap();
    OTW::serialised_vec(Msg::GetConfig, config.clone()).unwrap_err();
    OTW::serialised_vec(Msg::GetConfig, stats.clone()).unwrap_err();
    OTW::serialised_vec(Msg::GetConfig, response.clone()).unwrap_err();

    OTW::serialised_vec(Msg::Result, empty.clone()).unwrap_err();
    OTW::serialised_vec(Msg::Result, config.clone()).unwrap_err();
    OTW::serialised_vec(Msg::Result, stats.clone()).unwrap_err();
    OTW::serialised_vec(Msg::Result, response.clone()).unwrap();
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

    let vec = OTW::serialised_vec(Msg::Stats, DataRef::Stats(&stats)).unwrap();
    println!("{:?}", vec);
    let otw = OTW::from_bytes(&vec).unwrap();
    assert_eq!(
        otw,
        OTW {
            msg: Msg::Stats,
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
            msg: Msg::Result,
            data: Data::Result(Response::Ok)
        }
    )
}

pub const MAX_DUTY_PERCENT: f32 = 100.0;
pub const MIN_DUTY_PERCENT: f32 = 15.0; // 10% usually when a pwm fan starts to spin
pub const MIN_TEMP: f32 = 15.0;
pub const MAX_TEMP: f32 = 50.0;

#[test]
fn should_test_fixed() {
    let max_duty_percent: Fixed = Fixed::from_bits(1600);
    println!("MAX_DUTY_PERCENT: {:?}", max_duty_percent.to_bits());
    assert_eq!(max_duty_percent, Fixed::from_num(MAX_DUTY_PERCENT));

    let min_duty_percent: Fixed = Fixed::from_bits(240);
    println!("MIN_DUTY_PERCENT: {:?}", min_duty_percent.to_bits());
    assert_eq!(min_duty_percent, Fixed::from_num(MIN_DUTY_PERCENT));

    let min_temp: Fixed = Fixed::from_bits(240);
    println!("MIN_TEMP: {:?}", min_temp.to_bits());
    assert_eq!(min_temp, Fixed::from_num(MIN_TEMP));

    let max_temp: Fixed = Fixed::from_bits(800);
    println!("MAX_TEMP: {:?}", max_temp.to_bits());
    assert_eq!(max_temp, Fixed::from_num(MAX_TEMP));
}
