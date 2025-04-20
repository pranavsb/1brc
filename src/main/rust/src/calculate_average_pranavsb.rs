use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)] {
            println!($($arg)*);
        }
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    debug_println!("Args: {:?}", args);
    debug_println!("Working dir: {:?}", env::current_dir());
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "../../test/resources/samples/measurements-1.txt"
    };
    debug_println!("Filename: {}", filename);

    let file = match File::open(filename) {
        Ok(file) => file,
        Err(err) => panic!("Error opening file {}: {}", filename, err),
    };

    let (min_temps, max_temps, sum_temps, count_temps) = populate_raw_data(file);

    let avg_temps = calculate_average(sum_temps, count_temps);

    debug_println!("Min temps: {:?}", min_temps);
    debug_println!("Max temps: {:?}", max_temps);
    debug_println!("Avg temps: {:?}", avg_temps);

    let outfile = filename.split("/").last().expect("badly formatted filename");
    write_output_file(outfile, avg_temps, min_temps, max_temps);
}

fn populate_raw_data(file: File) -> (
    HashMap<String, f64>,
    HashMap<String, f64>,
    HashMap<String, f64>,
    HashMap<String, f64>,
) {
    let mut min_temps: HashMap<String, f64> = HashMap::new();
    let mut max_temps: HashMap<String, f64> = HashMap::new();
    let mut sum_temps: HashMap<String, f64> = HashMap::new();
    // use f64 so we can divide without converting while calculating avg
    let mut count_temps: HashMap<String, f64> = HashMap::new();

    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Some((city, temp_str)) = line.expect("IO error").split_once(";") {
            let temp = temp_str.parse::<f64>().expect("float parsing error");
            min_temps
                .entry(city.to_string())
                .and_modify(|e| *e = f64::min(*e, temp))
                .or_insert(temp);

            max_temps
                .entry(city.to_string())
                .and_modify(|e| *e = f64::max(*e, temp))
                .or_insert(temp);

            sum_temps
                .entry(city.to_string())
                .and_modify(|e| *e = *e + temp)
                .or_insert(temp);

            count_temps
                .entry(city.to_string())
                .and_modify(|e| *e = *e + 1.0)
                .or_insert(1.0);
        }
    }
    return (min_temps, max_temps, sum_temps, count_temps);
}

fn calculate_average(mut sum_temps: HashMap<String, f64>, count_temps: HashMap<String, f64>) -> HashMap<String, f64> {
    sum_temps.iter_mut().for_each(|(city, sum)| {
        let count = count_temps.get(city).expect("count not found");
        *sum = *sum / *count;
    });
    sum_temps
}

fn write_output_file(filename: &str, avg_temps: HashMap<String, f64>, max_temps: HashMap<String, f64>, min_temps: HashMap<String, f64>) {
    let mut cities = avg_temps.keys().cloned().collect::<Vec<String>>();
    cities.sort();

    let mut output_map: HashMap<String, String> = HashMap::new();
    for city in cities.iter() {
        let val = format!("{:.1}/{:.1}/{:.1}", min_temps.get(city).unwrap(), avg_temps.get(city).unwrap(), max_temps.get(city).unwrap());
        output_map.insert(city.to_string(), val);
    }

    let mut outfile = File::create(format!("./out/{}", filename)).expect("could not create output file");
    outfile.write_all("{".as_bytes()).expect("could not write to output file");
    for (i, (city, val)) in output_map.iter().enumerate() {
        if i == cities.len() - 1 {
            outfile.write_all(format!("{}={}", city, val).as_bytes()).expect("could not write to output file");
            break;
        }
        outfile.write_all(format!("{}={}, ", city, val).as_bytes()).expect("could not write to output file");
    }
    outfile.write_all("}".as_bytes()).expect("could not write to output file");
}

