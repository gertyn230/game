//! Система частинок для візуальних ефектів
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum ParticleType {
    Fire,
    Smoke,
    Magic,
    Hit,
    Heal,
    Sparkle,
    Dust,
    Blood,
}

#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: f32,
    pub alpha: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub particle_type: ParticleType,
}

impl Particle {
    pub fn new(x: f32, y: f32, particle_type: ParticleType) -> Self {
        let (vx, vy, size, life, color) = match particle_type {
            ParticleType::Fire => (
                rand::gen_range(-0.5, 0.5),
                rand::gen_range(-2.0, -1.0),
                rand::gen_range(3.0, 8.0),
                rand::gen_range(0.5, 1.5),
                Color::from_rgba(255, rand::gen_range(100, 200), 0, 255),
            ),
            ParticleType::Smoke => (
                rand::gen_range(-0.3, 0.3),
                rand::gen_range(-1.0, -0.5),
                rand::gen_range(5.0, 12.0),
                rand::gen_range(1.0, 2.0),
                Color::from_rgba(100, 100, 100, 150),
            ),
            ParticleType::Magic => (
                rand::gen_range(-1.0, 1.0),
                rand::gen_range(-1.0, 1.0),
                rand::gen_range(2.0, 5.0),
                rand::gen_range(0.5, 1.0),
                Color::from_rgba(150, 100, 255, 255),
            ),
            ParticleType::Hit => (
                rand::gen_range(-2.0, 2.0),
                rand::gen_range(-2.0, 0.0),
                rand::gen_range(2.0, 4.0),
                rand::gen_range(0.3, 0.6),
                Color::from_rgba(255, 255, 0, 255),
            ),
            ParticleType::Heal => (
                rand::gen_range(-0.5, 0.5),
                rand::gen_range(-1.5, -0.5),
                rand::gen_range(3.0, 6.0),
                rand::gen_range(0.8, 1.5),
                Color::from_rgba(100, 255, 100, 255),
            ),
            ParticleType::Sparkle => (
                rand::gen_range(-0.8, 0.8),
                rand::gen_range(-0.8, 0.8),
                rand::gen_range(1.5, 3.0),
                rand::gen_range(0.4, 0.8),
                Color::from_rgba(255, 255, 200, 255),
            ),
            ParticleType::Dust => (
                rand::gen_range(-0.5, 0.5),
                rand::gen_range(-0.3, 0.3),
                rand::gen_range(2.0, 5.0),
                rand::gen_range(0.5, 1.0),
                Color::from_rgba(200, 180, 150, 180),
            ),
            ParticleType::Blood => (
                rand::gen_range(-1.5, 1.5),
                rand::gen_range(-1.0, 0.5),
                rand::gen_range(2.0, 4.0),
                rand::gen_range(0.4, 0.8),
                Color::from_rgba(180, 0, 0, 255),
            ),
        };

        Particle {
            x,
            y,
            vx,
            vy,
            size,
            alpha: 1.0,
            life,
            max_life: life,
            color,
            particle_type,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.x += self.vx;
        self.y += self.vy;

        match self.particle_type {
            ParticleType::Fire | ParticleType::Smoke => {
                self.vy -= 0.05; // Піднімається вгору
                self.size += 0.1; // Розширюється
            }
            ParticleType::Blood | ParticleType::Dust => {
                self.vy += 0.1; // Гравітація
            }
            _ => {}
        }

        self.life -= dt;
        self.alpha = (self.life / self.max_life).clamp(0.0, 1.0);

        self.life > 0.0
    }

    pub fn draw(&self) {
        let mut color = self.color;
        color.a = self.alpha;

        match self.particle_type {
            ParticleType::Sparkle => {
                // Зірочка
                let points = 5;
                for i in 0..points {
                    let angle = (i as f32 * std::f32::consts::TAU / points as f32) + self.life;
                    let x2 = self.x + angle.cos() * self.size;
                    let y2 = self.y + angle.sin() * self.size;
                    draw_line(self.x, self.y, x2, y2, 1.5, color);
                }
            }
            _ => {
                draw_circle(self.x, self.y, self.size, color);
            }
        }
    }
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }

    pub fn emit(&mut self, x: f32, y: f32, particle_type: ParticleType, count: usize) {
        for _ in 0..count {
            self.particles.push(Particle::new(x, y, particle_type));
        }
    }

    pub fn emit_burst(&mut self, x: f32, y: f32, particle_type: ParticleType) {
        let count = match particle_type {
            ParticleType::Fire => 5,
            ParticleType::Smoke => 3,
            ParticleType::Magic => 10,
            ParticleType::Hit => 8,
            ParticleType::Heal => 12,
            ParticleType::Sparkle => 15,
            ParticleType::Dust => 6,
            ParticleType::Blood => 10,
        };
        self.emit(x, y, particle_type, count);
    }

    pub fn update(&mut self, dt: f32) {
        self.particles.retain_mut(|p| p.update(dt));
    }

    pub fn draw(&self) {
        for particle in &self.particles {
            particle.draw();
        }
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn count(&self) -> usize {
        self.particles.len()
    }
}
