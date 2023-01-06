#[derive(Copy, Clone, Debug, Default)]
pub struct RpmData {
    pub pump: f32,
    pub fan1: f32,
    pub fan2: f32,
    pub fan3: f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TempData {
    pub liq_in: f32,
    pub liq_out: f32,
    pub ambient: f32,
}
