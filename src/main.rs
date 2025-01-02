use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use ringbuf::{Producer, RingBuffer};
use std::sync::{Arc, Mutex};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const VISUALIZATION_SAMPLES: usize = 200;

fn main() {
    nannou::app(model).run();
}

struct Model {
    in_stream: audio::Stream<InputModel>,
    v_data: Arc<Mutex<Vec<f32>>>,
}

struct InputModel {
    pub producer: Producer<f32>,
    pub v_data: Arc<Mutex<Vec<f32>>>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .title("Audio Input")
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

       // Initialise the audio host so we can spawn an audio stream.

    let audio_host = audio::Host::new();
     // Create a ring buffer and split it into producer and consumer
    let latency_samples = 1024;
    let ring_buffer = RingBuffer::<f32>::new(latency_samples * 2);
    let (mut prod, _cons) = ring_buffer.split();
    
    for _ in 0..latency_samples {
          // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        prod.push(0.0).unwrap();
    }

    let v_data = Arc::new(Mutex::new(vec![0.0; VISUALIZATION_SAMPLES]));
    let v_data_clone = Arc::clone(&v_data);
 // Create input model and input stream using that model
    let in_model = InputModel { producer: prod, v_data };
    let in_stream = audio_host
        .new_input_stream(in_model)
        .capture(pass_in)
        .build()
        .unwrap();

    in_stream.play().unwrap();

    Model {
        in_stream,
        v_data: v_data_clone,
    }
}

fn pass_in(model: &mut InputModel, buffer: &Buffer) {
    for frame in buffer.frames() {
        for sample in frame {
            model.producer.push(*sample).ok();

            if let Ok(mut data) = model.v_data.try_lock() {
                if !data.is_empty() { 
                    data.remove(0);
                    data.push(*sample);
                }
            }
        }
    }
}

fn calculate_visual_dimensions(model: &Model) -> Option<Vec<Point2>> {
    if let Ok(data) = model.v_data.try_lock() {
        if data.is_empty() {
            return None;
        }

        let mut coordinates = Vec::with_capacity(VISUALIZATION_SAMPLES);
        let mut index = 0.0;
        let width_per_step = WINDOW_WIDTH as f32 / VISUALIZATION_SAMPLES as f32;

        for sample in data.iter() {
            // centerline + sample * height / 2
            let x = (index * width_per_step) - WINDOW_WIDTH as f32 / 2.0;
            let y = sample * WINDOW_HEIGHT as f32 / 2.0;
            coordinates.push(pt2(x, y));
            index += 1.0;
        }

        Some(coordinates)
    } else {
        None
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            if model.in_stream.is_paused() {
                model.in_stream.play().unwrap();
            } else {
                model.in_stream.pause().unwrap();
            }
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    

    draw.background().color(BLACK);


    if let Some(coordinates) = calculate_visual_dimensions(model) {
        if !coordinates.is_empty() {

            draw.polyline()
            .weight(2.0)
            .join_round()
            .points_colored(coordinates.iter().map(|&p| (p, STEELBLUE)));

   
        }
    }

    draw.to_frame(app, &frame).unwrap();
}