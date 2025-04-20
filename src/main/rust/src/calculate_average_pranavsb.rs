use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

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

    let outfile = get_output_filename(filename);
    debug_println!("Output file: {}", outfile);
    write_output_file(&outfile, avg_temps, min_temps, max_temps);

    // validate output if running debug build
    debug_validate!(&outfile);
}

fn populate_raw_data(
    file: File,
) -> (
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

fn calculate_average(
    mut sum_temps: HashMap<String, f64>,
    count_temps: HashMap<String, f64>,
) -> HashMap<String, f64> {
    sum_temps.iter_mut().for_each(|(city, sum)| {
        let count = count_temps.get(city).expect("count not found");
        *sum = *sum / *count;
    });
    sum_temps
}

fn write_output_file(
    filename: &str,
    avg_temps: HashMap<String, f64>,
    min_temps: HashMap<String, f64>,
    max_temps: HashMap<String, f64>,
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
            println!("Output file matches expected output");
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
