use log::*;
use nalgebra_glm::*;
use palette::{Gradient, LinSrgb};

const DAWN_START_TIME: f32 = 0.2;
const DAY_START_TIME: f32 = 0.3;
const DUSK_START_TIME: f32 = 0.7;
const NIGHT_START_TIME: f32 = 0.8;

#[derive(Copy, Clone, Debug)]
pub enum DayPhase {
    Night,
    Dawn,
    Day,
    Dusk,
}

pub struct DayNight {
    day_duration_s: f32,
    time: f32,
    day: u32,
    phase: DayPhase,
    light_angle: Vec3,
    sun_gradient: Gradient<LinSrgb>,
    light_gradient: Gradient<LinSrgb>,
    sky_gradient: Gradient<LinSrgb>,
    fog_gradient: Gradient<LinSrgb>,
    ambient_gradient: Gradient<LinSrgb>,
    sunlight_strength: f32,
    ambient_strength: f32,
}

impl DayNight {
    pub fn new(day_duration_s: f32) -> Self {
        assert!(day_duration_s > 0.0);

        let night_sun = LinSrgb::new(0.0, 0.0, 0.0);
        let dawn_sun = LinSrgb::new(1.0, 0.5, 0.4);
        let day_sun = LinSrgb::new(1.0, 0.6, 0.05);

        let night_light = LinSrgb::new(0.0, 0.0, 0.0);
        let dawn_light = LinSrgb::new(0.75, 0.3, 0.5);
        let day_light = LinSrgb::new(1.0, 0.95, 0.9);

        let night_sky = LinSrgb::new(0.0, 0.0, 0.0);
        let day_sky = LinSrgb::new(0.2, 0.4, 0.8);

        let night_fog = LinSrgb::new(0.01, 0.01, 0.02);
        let dawn_fog = LinSrgb::new(0.3, 0.1, 0.15);
        let day_fog = LinSrgb::new(0.25, 0.25, 0.35);

        let night_ambient = LinSrgb::new(0.2, 0.2, 0.3);
        let dawn_ambient = LinSrgb::new(0.3, 0.1, 0.2);
        let day_ambient = LinSrgb::new(0.3, 0.3, 0.27);

        let sun_gradient = Gradient::with_domain(vec![
            (0.0, night_sun),
            (DAWN_START_TIME, night_sun),
            (
                DAWN_START_TIME + (DAY_START_TIME - DAWN_START_TIME) / 2.0,
                dawn_sun,
            ),
            (DAY_START_TIME, day_sun),
            (DUSK_START_TIME, day_sun),
            (
                DUSK_START_TIME + (NIGHT_START_TIME - DUSK_START_TIME) / 2.0,
                dawn_sun,
            ),
            (NIGHT_START_TIME, night_sun),
            (1.0, night_sun),
        ]);
        let light_gradient = Gradient::with_domain(vec![
            (0.0, night_light),
            (DAWN_START_TIME, night_light),
            (
                DAWN_START_TIME + (DAY_START_TIME - DAWN_START_TIME) / 2.0,
                dawn_light,
            ),
            (DAY_START_TIME, day_light),
            (DUSK_START_TIME, day_light),
            (
                DUSK_START_TIME + (NIGHT_START_TIME - DUSK_START_TIME) / 2.0,
                dawn_light,
            ),
            (NIGHT_START_TIME, night_light),
            (1.0, night_light),
        ]);
        let sky_gradient = Gradient::with_domain(vec![
            (0.0, night_sky),
            (DAWN_START_TIME, night_sky),
            (DAY_START_TIME, day_sky),
            (DUSK_START_TIME, day_sky),
            (NIGHT_START_TIME, night_sky),
            (1.0, night_light),
        ]);
        let fog_gradient = Gradient::with_domain(vec![
            (0.0, night_fog),
            (DAWN_START_TIME, night_fog),
            (
                DAWN_START_TIME + (DAY_START_TIME - DAWN_START_TIME) / 2.0,
                dawn_fog,
            ),
            (DAY_START_TIME, day_fog),
            (DUSK_START_TIME, day_fog),
            (
                DUSK_START_TIME + (NIGHT_START_TIME - DUSK_START_TIME) / 2.0,
                dawn_fog,
            ),
            (NIGHT_START_TIME, night_fog),
            (1.0, night_fog),
        ]);
        let ambient_gradient = Gradient::with_domain(vec![
            (0.0, night_ambient),
            (DAWN_START_TIME, night_ambient),
            (
                DAWN_START_TIME + (DAY_START_TIME - DAWN_START_TIME) / 2.0,
                dawn_ambient,
            ),
            (DAY_START_TIME, day_ambient),
            (DUSK_START_TIME, day_ambient),
            (
                DUSK_START_TIME + (NIGHT_START_TIME - DUSK_START_TIME) / 2.0,
                dawn_ambient,
            ),
            (NIGHT_START_TIME, night_ambient),
            (1.0, night_ambient),
        ]);
        DayNight {
            day_duration_s,
            time: 0.0,
            day: 1,
            phase: DayPhase::Night,
            light_angle: Vec3::z(),
            sun_gradient,
            light_gradient,
            sky_gradient,
            fog_gradient,
            ambient_gradient,
            sunlight_strength: 1.0,
            ambient_strength: 0.1,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.time = self.time + delta / self.day_duration_s;
        while self.time > 1.0 {
            self.time = self.time - 1.0;
            self.day = self.day + 1;
        }
        self.update_data();
    }

    fn update_data(&mut self) {
        self.phase = if self.time >= NIGHT_START_TIME || self.time < DAWN_START_TIME {
            DayPhase::Night
        } else if self.time >= DUSK_START_TIME {
            DayPhase::Dusk
        } else if self.time >= DAY_START_TIME {
            DayPhase::Day
        } else {
            DayPhase::Dawn
        };
        let angle = self.time * std::f32::consts::PI * 2.0 - std::f32::consts::PI / 2.0;
        self.light_angle = Vec3::new(angle.cos(), 0.1, angle.sin());
    }

    pub fn get_light_angle(&self) -> Vec3 {
        self.light_angle
    }

    pub fn get_time(&self) -> f32 {
        self.time
    }

    pub fn set_time(&mut self, time: f32) {
        if time < 0.0 || time > 1.0 {
            warn!(
                "Time {} was outside of allowed range, resetting to 0.3",
                time
            );
            self.time = 0.3;
        } else {
            self.time = time;
        }
        self.update_data();
    }

    pub fn get_phase(&self) -> DayPhase {
        self.phase
    }

    pub fn get_day(&self) -> u32 {
        self.day
    }

    pub fn get_sun_color(&self) -> Vec3 {
        let color = self.sun_gradient.get(self.time);
        Vec3::new(color.red, color.green, color.blue)
    }

    pub fn get_light_color(&self) -> Vec3 {
        let color = self.light_gradient.get(self.time);
        Vec3::new(color.red, color.green, color.blue) * self.sunlight_strength
    }

    pub fn get_sky_color(&self) -> Vec3 {
        let color = self.sky_gradient.get(self.time);
        Vec3::new(color.red, color.green, color.blue)
    }

    pub fn get_fog_color(&self) -> Vec3 {
        let color = self.fog_gradient.get(self.time);
        Vec3::new(color.red, color.green, color.blue)
    }

    pub fn get_ambient_color(&self) -> Vec3 {
        let color = self.ambient_gradient.get(self.time);
        Vec3::new(color.red, color.green, color.blue) * self.ambient_strength
    }
}
