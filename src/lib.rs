
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

struct Velocity
{
    x: f64,
    y: f64
}

struct Particle
{
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    alpha: f64,
    velocity: Velocity
}

fn rand() -> f64
{
    let mut rands = [0u8, 2];
    window().crypto().unwrap().get_random_values_with_u8_array(&mut rands).unwrap();
    
    let sign = if rands[1] > 127 {
        -1.
    }
    else
    {
        1.
    };

    rands[0] as f64  / 256. * sign
}

impl Particle
{
    fn new() -> Particle
    {
        let mut size = rand() * 25.;
        if size < 0.
        {
            size *= -1.;
        }

        let dx = rand() * 5.;
        let mut dy = rand() * -10.;
        if dy > 0.
        {
            dy *= -1.;
        }

        Particle{
            x:375.,
            y:500.,
            width: size,
            height: size,
            alpha: 1.,
            velocity: Velocity{x: dx, y: dy}
        }
    }

    fn update(&mut self)
    {
        self.x += self.velocity.x;
        self.y += self.velocity.y;
        self.velocity.y += 0.1; // gravitational accel
        self.alpha -= 0.01;
    }
}

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    // Here we want to call `requestAnimationFrame` in a loop, but only a fixed
    // number of times. After it's done we want all our resources cleaned up. To
    // achieve this we're using an `Rc`. The `Rc` will eventually store the
    // closure we want to execute on each frame, but to start out it contains
    // `None`.
    //
    // After the `Rc` is made we'll actually create the closure, and the closure
    // will reference one of the `Rc` instances. The other `Rc` reference is
    // used to store the closure, request the first frame, and then is dropped
    // by this function.
    //
    // Inside the closure we've got a persistent `Rc` reference, which we use
    // for all future iterations of the loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let document = window().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let mut particles = Vec::<Particle>::new();
    particles.push(Particle::new());

    let sea_y = 550.;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {

        context.set_fill_style(&JsValue::from_str("white")); // clear screen
        context.fill_rect(0., 0., canvas.width() as f64, canvas.height() as f64); // clear screen

        context.set_fill_style(&JsValue::from_str("lightblue")); // sea
        context.fill_rect(0., sea_y, canvas.width() as f64, canvas.height() as f64 - sea_y); // sea

        if rand() > 0.5 && particles.len() < 500
        {
            particles.push(Particle::new());
        }

        let mut particles_to_remove = Vec::new();
        for i in 0..particles.len()
        {
            context.set_fill_style(&JsValue::from_str(&format!("rgba({},{},{}, {})", 0xdc, 0x14, 0x3c, particles[i].alpha)[..]));
            context.fill_rect(particles[i].x, particles[i].y, particles[i].width, particles[i].height);
            particles[i].update();
            // web_sys::console::log_1(&JsValue::from_str(&format!("Particle {} at x: {}, y: {}", i, particles[i].x, particles[i].y)[..]));
            if particles[i].y > sea_y && particles.len() > i
            {
                // particles.remove(i);
                particles_to_remove.push(i);
                particles.push(Particle::new());
            }
        }

        for i in particles_to_remove
        {
            particles.remove(i);
        }

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
