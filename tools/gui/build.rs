use dotenvy::from_filename_iter;

fn main() {
    // Attempt to read the .env file
    if let Ok(iter) = from_filename_iter(".env") {
        for item in iter {
            match item {
                Ok((key, value)) => {
                    // Set the environment variable for the compiler
                    println!("cargo:rustc-env={}={}", key, value);
                }
                Err(err) => {
                    // Handle parsing errors (e.g., invalid lines in .env)
                    eprintln!("Warning: Failed to parse .env entry: {}", err);
                }
            }
        }
    } else {
        // .env file is missing; this is not an error
        println!("cargo:warning=.env file not found. Skipping environment variable loading.");
    }
}
