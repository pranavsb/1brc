use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::{env, u32};

macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)] {
            println!($($arg)*);
        }
    };
}

macro_rules! debug_validate {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)] {
            debug_validate($($arg)*);
        }
    };
}

#[derive(Debug)]
// message format sent from main to worker thread
struct ProcessingMessage {
    temp_sample: f64,
    // will be true only once all data is sent
    done: bool,
}

#[derive(Debug)]
// message format from worker to main (sent once)
struct ComputedMessage {
    min: f64,
    max: f64,
    avg: f64,
}

const WORKER_THREADS_COUNT: u32 = 500;

fn main() {
    let args: Vec<String> = env::args().collect();
    debug_println!("Args: {:?}", args);
    debug_println!("Working dir: {:?}", env::current_dir());
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "measurements.txt"
    };
    debug_println!("Filename: {}", filename);

    let file = match File::open(filename) {
        Ok(file) => file,
        Err(err) => panic!("Error opening file {}: {}", filename, err),
    };

    let (min_temps, max_temps, avg_temps) = calculate_average(file);

    debug_println!("Min temps: {:?}", min_temps);
    debug_println!("Max temps: {:?}", max_temps);
    debug_println!("Avg temps: {:?}", avg_temps);

    let outfile = get_output_filename(filename);
    debug_println!("Output file: {}", outfile);
    write_output_file(&outfile, min_temps, max_temps, avg_temps);

    // validate output if running debug build
    debug_validate!(&outfile);
}

fn calculate_average(
    file: File,
) -> (
    HashMap<String, f64>,
    HashMap<String, f64>,
    HashMap<String, f64>,
) {
    // using 3 maps instead of custom struct cuz I thought it might be slower (unlikely)
    let mut min_temps: HashMap<String, f64> = HashMap::new();
    let mut max_temps: HashMap<String, f64> = HashMap::new();
    let mut avg_temps: HashMap<String, f64> = HashMap::new();

    // creating one channel per city
    // we know the 1b dataset has ~400 so creating 500 no matter what

    // TODO preallocate all datastructures based on 1b file
    // city to channel map
    let mut city_to_worker_index: HashMap<String, usize> = HashMap::new();
    // to send Message with a sample
    let mut main_to_worker_channels = Vec::new();
    // to signal "done"
    let mut worker_to_main_channels = Vec::new();

    for _i in 0..WORKER_THREADS_COUNT {
        let (worker_tx, worker_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();

        main_to_worker_channels.push(worker_tx);
        worker_to_main_channels.push(done_rx);

        // thread per city
        thread::spawn(move || {
            let mut sum = 0.0;
            // use f64 so we can divide without converting while calculating avg
            let mut count = 0.0;
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;

            loop {
                let msg: ProcessingMessage = worker_rx.recv().unwrap();
                if msg.done {
                    // worker thread was unused
                    if count < 1.0 {
                        done_tx
                            .send(ComputedMessage {
                                min: f64::NAN,
                                max: f64::NAN,
                                avg: f64::NAN,
                            })
                            .expect("Error sending computed msg");
                        break;
                    }
                    done_tx
                        .send(ComputedMessage {
                            min,
                            max,
                            avg: sum / count,
                        })
                        .expect("Error sending computed msg");
                    break;
                }

                sum += msg.temp_sample;
                count += 1.0;
                min = f64::min(min, msg.temp_sample);
                max = f64::max(max, msg.temp_sample);
            }
        });
    }

    let reader = BufReader::new(file);
    let mut next_worker_index = 0;
    for line in reader.lines() {
        if let Some((city, temp_str)) = line.expect("IO error").split_once(";") {
            let temp = temp_str.parse::<f64>().expect("float parsing error");
            if !city_to_worker_index.contains_key(city) {
                if next_worker_index + 1 > WORKER_THREADS_COUNT {
                    panic!("Not enough worker threads, need one thread per city");
                }
                city_to_worker_index
                    .insert(city.to_string(), next_worker_index.try_into().unwrap());
                next_worker_index += 1;
            }
            let msg = ProcessingMessage {
                temp_sample: temp,
                done: false,
            };
            let worker_index = city_to_worker_index.get(city).unwrap();
            main_to_worker_channels[*worker_index]
                .send(msg)
                .expect("Error sending message to worker");
        }
    }
    // send done msg to all workers
    // since processing is lightweight, we can assume it's already done
    // if not, i'd rather add a short sleep in the main thread than create channels in the reverse direction
    for worker_tx in main_to_worker_channels.iter() {
        let msg = ProcessingMessage {
            temp_sample: 0.0,
            done: true,
        };
        worker_tx.send(msg).expect("Error sending done msg");
    }

    // collect all computed messages``
    for (_i, (city, worker_index)) in city_to_worker_index.iter().enumerate() {
        let msg: ComputedMessage = worker_to_main_channels[*worker_index].recv().unwrap();
        debug_println!(
            "Computed message: {:?} for city {} with worker index {}",
            msg,
            city,
            worker_index
        );
        min_temps.insert(city.to_string(), msg.min);
        max_temps.insert(city.to_string(), msg.max);
        avg_temps.insert(city.to_string(), msg.avg);
    }
    return (min_temps, max_temps, avg_temps);
}

fn write_output_file(
    filename: &str,
    min_temps: HashMap<String, f64>,
    max_temps: HashMap<String, f64>,
    avg_temps: HashMap<String, f64>,
) {
    let mut cities = avg_temps.keys().cloned().collect::<Vec<String>>();
    cities.sort();

    let mut output_map: HashMap<String, String> = HashMap::new();
    for city in cities.iter() {
        let val = format!(
            "{:.1}/{:.1}/{:.1}",
            min_temps.get(city).unwrap(),
            (avg_temps.get(city).unwrap() * 10.0).ceil() / 10.0,
            max_temps.get(city).unwrap()
        );
        output_map.insert(city.to_string(), val);
    }

    let mut outfile =
        File::create(format!("./out/{}", filename)).expect("could not create output file");
    outfile
        .write_all("{".as_bytes())
        .expect("could not write to output file");
    for (i, city) in cities.iter().enumerate() {
        if i == cities.len() - 1 {
            outfile
                .write_all(format!("{}={}", city, output_map.get(city).unwrap()).as_bytes())
                .expect("could not write to output file");
            break;
        }
        outfile
            .write_all(format!("{}={}, ", city, output_map.get(city).unwrap()).as_bytes())
            .expect("could not write to output file");
    }
    outfile
        .write_all("}\n".as_bytes())
        .expect("could not write to output file");
}

// not doing anything fancy to avoid dependencies
// no need to show diff, we can manually compare both files
fn debug_validate(outfile: &str) {
    let mut expected_reader = BufReader::new(
        File::open(format!("../../test/resources/samples/{}", outfile))
            .expect("could not open expected output"),
    );
    let mut actual_reader = BufReader::new(
        File::open(format!("./out/{}", outfile)).expect("could not open output file"),
    );

    let mut buf_expected = [0; 1000];
    let mut buf_actual = [0; 1000];
    loop {
        let n_expected = expected_reader
            .read(&mut buf_expected)
            .expect("could not read expected output");
        let n_actual = actual_reader
            .read(&mut buf_actual)
            .expect("could not read output file");

        if n_expected == 0 && n_actual == 0 {
            println!("\n\nOutput file matches expected output!");
            break;
        } else if n_expected != n_actual {
            panic!("Output file does not match expected output");
        }
        if buf_expected == buf_actual {
            continue;
        }
        panic!("Output file does not match expected output");
    }
}

fn get_output_filename(filename: &str) -> String {
    let path = Path::new(filename);
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.split('.').next())
        .and_then(|name| Some(format!("{}.out", name)))
        .unwrap()
        .to_string()
}
