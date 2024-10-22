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

    let env_vars = [
        "TEST_MNEMONIC",
        "LOCAL_MNEMONIC",
        "LOCAL_CODE_ID_TASK_QUEUE",
        "LOCAL_CODE_ID_MOCK_OPERATORS",
        "LOCAL_CODE_ID_VERIFIER_SIMPLE",
        "LOCAL_CODE_ID_VERIFIER_ORACLE",
        "TEST_CODE_ID_TASK_QUEUE",
        "TEST_CODE_ID_MOCK_OPERATORS",
        "TEST_CODE_ID_VERIFIER_SIMPLE",
        "TEST_CODE_ID_VERIFIER_ORACLE",
    ];

    for var in &env_vars {
        println!("cargo:rerun-if-env-changed={}", var);
    }
}
