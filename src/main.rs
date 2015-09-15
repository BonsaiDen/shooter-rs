#[macro_use]
extern crate allegro;
extern crate allegro_primitives;
extern crate rand;
extern crate allegro_sys;

use allegro::*;
use allegro_primitives::*;
use rand::Rng;
use std::f32;


static COLORS: [(u8, u8, u8); 8] = [
    (0xf2, 0x00, 0x26), // Red
    (0xfd, 0x83, 0x1c), // Orange
    (0xfd, 0xda, 0x31), // Yellow
    (0x3c, 0xdc, 0x00), // Green
    (0x33, 0xd0, 0xd1), // Teal
    (0x0f, 0x5c, 0xf9), // Blue
    (0x82, 0x0c, 0xe6), // Purple
    (0xec, 0x34, 0xa7)  // Pink
];

fn rgb_to_hsl(color: (u8, u8, u8)) -> (f32, f32, f32) {

    let r = color.0 as f32 / 255.0;
    let g = color.1 as f32 / 255.0;
    let b = color.2 as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let mut h = 0.0;
    let mut s = 0.0;
    let l = (max + min) / 2.0;

    if max != min {

        let d = max - min;

        s = if l > 0.5 {
            d / (2.0 - max - min)

        } else {
            d / (max + min)
        };

        h = if r > g && r > b {
            (g - b) / d + if g < b {
                6.0
            } else {
                0.0
            }

        } else if g > b {
            (b - r) / d + 2.0

        } else {
            (r - g) / d + 4.0

        } / 6.0 * 360.0;

    }

    (h, s, l)

}

fn hsl_to_rgb(color: (f32, f32, f32)) -> (u8, u8, u8) {

    let (h, s, l) = color;


    let m2 = if l <= 0.5 { l*(s + 1.0) } else { l + s - l*s };
    let m1 = l*2.0 - m2;
    let h = h / 360.0;

    let mut r = 0;
    let mut g = 0;
    let mut b = 0;

    fn hue_to_rgb(m1: f32, m2: f32, h: f32) -> f32 {
        let h = if h < 0.0 { h + 1.0 } else if h > 1.0 { h - 1.0 } else { h };

        if 0.0 <= h && h < 1.0/6.0 {
            m1 + (m2 - m1)*h*6.0

        } else if 1.0/6.0 <= h && h < 1.0/2.0 {
            m2

        } else if 1.0/2.0 <= h && h < 2.0/3.0 {
            m1 + (m2 - m1)*(4.0 - 6.0*h)

        } else if 2.0/3.0 <= h && h <= 1.0 {
            m1

        } else {
            0.0
        }
    }

    r = (255.0 * hue_to_rgb(m1, m2, h + 1.0 / 3.0)).round() as u8;
    g = (255.0 * hue_to_rgb(m1, m2, h)).round() as u8;
    b = (255.0 * hue_to_rgb(m1, m2, h - 1.0 / 3.0)).round() as u8;

    (r, g, b)

}

fn brighten(color: (u8, u8, u8), by: f32) -> (u8, u8, u8) {
    let (h, s, l) = rgb_to_hsl(color);
    hsl_to_rgb((h, s, l * (1.0 + by)))
}

fn darken(color: (u8, u8, u8), by: f32) -> (u8, u8, u8) {
    let (h, s, l) = rgb_to_hsl(color);
    hsl_to_rgb((h, s, l * (1.0 - by)))
}

struct Ship {
    x: f32,
    y: f32,
    r: f32,
    mx: f32,
    my: f32,
    max_speed: f32,
    acceleration: f32,
    rotation: f32,
    color: u8
}

impl Ship {

    pub fn new(x: f32, y: f32, color: u8) -> Ship {
        Ship {
            x: x,
            y: y,
            r: 0.0,
            mx: 0.0,
            my: 0.0,
            max_speed: 3.0,
            acceleration: 4.0,
            rotation: 120.0,
            color: color
        }
    }

    pub fn step(&mut self, dt: f32, steer: f32, thrust: bool) {

        self.r += f32::consts::PI / 180.0 * self.rotation * dt * steer;

        if thrust {
            let ax = self.r.cos() * self.acceleration * dt;
            let ay = self.r.sin() * self.acceleration * dt;
            self.mx += ax;
            self.my += ay;
        }


        // Limit speed
        let mr = self.my.atan2(self.mx);
        let mut s = ((self.mx * self.mx) + (self.my * self.my)).sqrt();

        // Allow for easier full stop
        if s < 0.15 {
            s *= 0.95;
        }

        self.mx = mr.cos() * s.min(self.max_speed);
        self.my = mr.sin() * s.min(self.max_speed);
        self.x += self.mx;
        self.y += self.my;


    }

}

struct DrawableShip {
    ship: Ship,
    color_light: Color,
    color_mid: Color,
    color_dark: Color,
    scale: f32,
    particle_system: ParticleSystem
}

impl DrawableShip {

    pub fn new(core: &Core, mut ship: Ship) -> DrawableShip {
        let cl = COLORS[ship.color as usize];
        let cm = darken(COLORS[ship.color as usize], 0.5);
        let cd = darken(COLORS[ship.color as usize], 0.75);
        DrawableShip {
            ship: ship,
            color_light: core.map_rgb(cl.0, cl.1, cl.2),
            color_mid: core.map_rgb(cm.0, cm.1, cm.2),
            color_dark: core.map_rgb(cd.0, cd.1, cd.2),
            scale: 2.0,
            particle_system: ParticleSystem::new(50)
        }
    }

    pub fn step(&mut self, rng: &mut rand::XorShiftRng, dt: f32, steer: f32, thrust: bool) {

        if rng.gen::<u8>() > 20 && thrust {
            if let Some(p) = self.particle_system.get() {

                // Exhaust angle
                let w = 0.95;
                let mr = self.ship.my.atan2(self.ship.mx);
                let d = (self.ship.r - mr);

                // Increase engine velocity when flying backwards
                let mut dr = d.abs() % (f32::consts::PI * 2.0);
                if dr > f32::consts::PI  {
                    dr = f32::consts::PI * 2.0 - dr;
                }

                let cs = (1.0 - w) * mr.cos() + w * self.ship.r.cos();
                let sn = (1.0 - w) * mr.sin() + w * self.ship.r.sin();
                let mr = sn.atan2(cs) + f32::consts::PI;

                p.color = COLORS[self.ship.color as usize];
                p.x = self.ship.x + mr.cos() * 9.0 * self.scale + 0.5;
                p.y = self.ship.y + mr.sin() * 9.0 * self.scale + 0.5;
                p.s = 5.0;
                p.sms = -2.5;
                p.v = (86.0 + rng.gen::<u8>() as f32 / 9.0) * 0.5 + dr * 30.0;
                p.vms = 0.0;
                p.r = mr + ((rng.gen::<u8>() as f32) - 96.0) / 96.0;
                p.rms = ((rng.gen::<u8>() as f32) - 128.0) / 128.0;
                p.fadeout = 0.25;
                p.lifetime = 0.5;
                p.remaining = 0.5;

            }
        }

        self.ship.step(dt, steer, thrust);
        self.particle_system.step(dt);

    }

    pub fn draw(&mut self, core: &Core, prim: &PrimitivesAddon) {

        let light = self.color_light;
        let mid = self.color_mid;
        let scale = self.scale;

        self.draw_exhaust(prim, mid, scale, scale);
        self.draw_body(prim, light, scale, scale);
        self.draw_body(prim, mid, scale, scale * 0.66);

        prim.draw_circle(self.ship.x + 0.5, self.ship.y + 0.5, 11.0 * scale, self.color_dark, 0.5 * scale);
        self.particle_system.draw(&core, &prim);

        // Velocity vector
        let mr = self.ship.my.atan2(self.ship.mx);
        let s = ((self.ship.mx * self.ship.mx) + (self.ship.my * self.ship.my)).sqrt() * 10.0;
        let tx = self.ship.x + mr.cos() * s;
        let ty = self.ship.y + mr.sin() * s;
        prim.draw_line(self.ship.x + 0.5, self.ship.y + 0.5, tx + 0.5, ty + 0.5, light, 1.0);


        let w = 0.5;
        let mr = self.ship.my.atan2(self.ship.mx);
        let cs = (1.0 - w) * mr.cos() + w * self.ship.r.cos();
        let sn = (1.0 - w) * mr.sin() + w * self.ship.r.sin();
        let mc = sn.atan2(cs) + f32::consts::PI;
        let s = 50.0;
        let tx = self.ship.x + mc.cos() * s;
        let ty = self.ship.y + mc.sin() * s;
        prim.draw_line(self.ship.x + 0.5, self.ship.y + 0.5, tx + 0.5, ty + 0.5, light, 1.0);

    }

    fn draw_body(&mut self, prim: &PrimitivesAddon, color: Color, base_scale: f32, body_scale: f32) {

        let beta = f32::consts::PI / (2 as f32).sqrt();
        let ox = self.ship.r.cos() * -2.0 * base_scale + 0.5;
        let oy = self.ship.r.sin() * -2.0 * base_scale + 0.5;

        let ax = ox + self.ship.x + self.ship.r.cos() * 12.0 * body_scale;
        let ay = oy + self.ship.y + self.ship.r.sin() * 12.0 * body_scale;

        let bx = ox + self.ship.x + (self.ship.r + beta).cos() * 9.0 * body_scale;
        let by = oy + self.ship.y + (self.ship.r + beta).sin() * 9.0 * body_scale;

        let cx = ox + self.ship.x + (self.ship.r - beta).cos() * 9.0 * body_scale;
        let cy = oy + self.ship.y + (self.ship.r - beta).sin() * 9.0 * body_scale;

        prim.draw_triangle(ax, ay, bx, by, cx, cy, color, 0.5 * body_scale);

    }

    fn draw_exhaust(&mut self, prim: &PrimitivesAddon, color: Color, base_scale: f32, body_scale: f32) {

        let beta = f32::consts::PI / 1.15;
        let ox = self.ship.r.cos() * -2.0 * base_scale + 0.5;
        let oy = self.ship.r.sin() * -2.0 * base_scale + 0.5;

        let ax = ox + self.ship.x - self.ship.r.cos() * 8.0 * body_scale;
        let ay = oy + self.ship.y - self.ship.r.sin() * 8.0 * body_scale;

        let bx = ox + self.ship.x + (self.ship.r + beta).cos() * 6.0 * body_scale;
        let by = oy + self.ship.y + (self.ship.r + beta).sin() * 6.0 * body_scale;

        let cx = ox + self.ship.x + (self.ship.r - beta).cos() * 6.0 * body_scale;
        let cy = oy + self.ship.y + (self.ship.r - beta).sin() * 6.0 * body_scale;

        prim.draw_triangle(ax, ay, bx, by, cx, cy, color, 0.5 * body_scale);

    }

}

struct Particle {

    active: bool,

    color: (u8, u8, u8),

    // Position
    x: f32,
    y: f32,

    // Velocity
    v: f32,

    // Size
    s: f32,

    // Size modification per second
    sms: f32,

    // Velocity modification per seond
    vms: f32,

    // Angle
    r: f32,

    // Angle modification per second
    rms: f32,

    fadeout: f32,
    lifetime: f32,
    remaining: f32

}

impl Particle {

    fn is_active(&mut self) -> bool {
        self.active
    }

    fn step(&mut self, dt: f32) {
        if self.remaining <= 0.0 {
            self.active = false;

        } else {
            self.x += self.r.cos() * self.v * dt;
            self.y += self.r.sin() * self.v * dt;
            self.s += self.sms * dt;
            self.r += self.rms * dt;
            self.v += self.vms * dt;
            self.remaining -= dt;
        }
    }

    fn draw(&mut self, core: &Core, prim: &PrimitivesAddon) {

        let lp = 1.0 / self.lifetime * self.remaining;
        let alpha = if lp <= self.fadeout {
            1.0 / (self.lifetime * self.fadeout) * self.remaining.max(0.0)

        } else {
            1.0
        };

        let hs = self.s / 2.0;
        let color = core.map_rgb((self.color.0 as f32 * alpha) as u8, (self.color.1 as f32* alpha) as u8, (self.color.2 as f32* alpha) as u8);
        prim.draw_filled_rectangle(self.x - hs + 0.5, self.y - hs + 0.5, self.x + hs + 0.5, self.y + hs + 0.5, color);

    }

}

struct ParticleSystem {
    max_particles: usize,
    particles: Vec<Particle>,
}

impl ParticleSystem {

    pub fn new(max_particles: usize) -> ParticleSystem {

        let mut particles = vec![];
        for i in 0..max_particles {
            particles.push(Particle {
                active: false,
                color: (0, 0, 0),
                x: 0.0,
                y: 0.0,
                s: 1.0,
                sms: 0.0,
                v: 0.0,
                vms: 0.0,
                r: 0.0,
                rms: 0.0,
                fadeout: 0.0,
                lifetime: 0.0,
                remaining: 0.0
            });
        }

        ParticleSystem {
            max_particles: max_particles,
            particles: particles
        }

    }

    pub fn get(&mut self) -> Option<&mut Particle> {

        for p in self.particles.iter_mut() {
            if !p.is_active() {
                p.active = true;
                //p.color = COLORS[color as usize];
                p.x = 0.0;
                p.y = 0.0;
                p.s = 5.0;
                p.sms = -2.5;
                p.v = 0.0;//(16.0 + rng.gen::<u8>() as f32 / 9.0) * 2.0;
                p.vms = 0.0;
                p.r = 0.0;//((rng.gen::<u8>() as f32) - 128.0) / 128.0 / 4.0;
                p.rms = 0.0;//((rng.gen::<u8>() as f32) - 128.0) / 128.0;
                p.fadeout = 0.25;
                p.lifetime = 0.8;
                p.remaining = 0.8;
                return Some(p);
            }
        }

        None

    }

    pub fn step(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            if p.is_active() {
                p.step(dt);
            }
        }
    }

    pub fn draw(&mut self, core: &Core, prim: &PrimitivesAddon) {
        for p in self.particles.iter_mut() {
            if p.is_active() {
                p.draw(core, prim);
            }
        }
    }

}


allegro_main! {

    let mut core = Core::init().unwrap();

    core.set_new_display_option(DisplayOption::SampleBuffers, 1, DisplayOptionImportance::Suggest);
    core.set_new_display_option(DisplayOption::Samples, 8, DisplayOptionImportance::Suggest);

    let disp = Display::new(&core, 720, 540).unwrap();
	disp.set_window_title("Rust example");

	core.install_keyboard().unwrap();
	core.install_mouse().unwrap();

    let prim = PrimitivesAddon::init(&core).unwrap();

    let timer = Timer::new(&core, 1.0 / 60.0).unwrap();

    let q = EventQueue::new(&core).unwrap();
	q.register_event_source(disp.get_event_source());
	q.register_event_source(core.get_keyboard_event_source());
	q.register_event_source(core.get_mouse_event_source());
	q.register_event_source(timer.get_event_source());

    timer.start();

    let back_color = core.map_rgb_f(0.0, 0.0, 0.0);

    let mut players = vec![
        DrawableShip::new(&core, Ship::new(60.0, 60.0, 0)),
        DrawableShip::new(&core, Ship::new(140.0, 60.0, 1)),
        DrawableShip::new(&core, Ship::new(220.0, 60.0, 2)),
        DrawableShip::new(&core, Ship::new(300.0, 60.0, 3)),
        DrawableShip::new(&core, Ship::new(60.0, 140.0, 4)),
        DrawableShip::new(&core, Ship::new(140.0, 140.0, 5)),
        DrawableShip::new(&core, Ship::new(220.0, 140.0, 6)),
        DrawableShip::new(&core, Ship::new(300.0, 140.0, 7)),
    ];

    let mut rng = rand::XorShiftRng::new_unseeded();
    let mut key_state: [bool; 255] = [false; 255];

    let mut redraw = true;
    'exit: loop {

        if redraw && q.is_empty() {

            core.clear_to_color(back_color);

            for p in players.iter_mut() {
                p.draw(&core, &prim);
            }

            disp.flip();
            redraw = false;

        }

        match q.wait_for_event() {

			DisplayClose{source: src, ..} => {
				assert!(disp.get_event_source().get_event_source() == src);
				println!("Display close event...");
				break 'exit;
			},

			KeyDown{keycode: k, ..} if (k as u32) < 255 => {
                key_state[k as usize] = true;
                println!("{}", k as u32);
			},

			KeyUp{keycode: k, ..} if (k as u32) < 255 => {
                key_state[k as usize] = false;
			},

            TimerTick{..} => {

                let dt = 1.0 / 60.0;

                for (i, p) in players.iter_mut().enumerate() {

                    let mut steer = 0.0;
                    let mut thrust = false;

                    if i == 0 {

                        if key_state[1] {
                            steer -= 1.0;
                        }

                        if key_state[4] {
                            steer += 1.0;
                        }

                        thrust = key_state[23];

                    }

                    p.step(&mut rng, dt, steer, thrust);

                }

                redraw = true

            },

	        MouseButtonDown{button: b, ..} => {
				println!("Mouse button {} pressed", b);
			},

            _ => () // println!("Some other event...")

        }

    }

}

