mod runner;
mod sample;

use anyhow::anyhow;
use sample::Test;
use std::fs;

fn read_test_from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Test> {
    let file = fs::File::open(path)?;
    let u = serde_json::from_reader(file)?;
    Ok(u)
}

pub fn main() -> anyhow::Result<()> {
    let mut tests = Vec::new();

    for f in std::env::args().skip(1) {
        if fs::metadata(&f).map(|m| m.is_dir()).unwrap_or_else(|e| {
            println!("Error accessing {}: {}", f, e);
            false
        }) {
            continue;
        }

        tests.push(read_test_from_file(&f)?);
    }

    let total_fixtures: usize = tests.iter().map(|t| t.fixtures.len()).sum();
    let mut total_failed = 0i32;

    println!("Total fixtures: {}", total_fixtures);

    for test in tests {
        for fixture_no in 0..test.fixtures.len() {
            let output = match runner::init_fixture(&test, fixture_no) {
                Ok(initialized_fixture) => {
                    match runner::run(initialized_fixture, test.fixtures[fixture_no].expected.step)
                    {
                        Ok(final_state) => {
                            let mut res = format!("Ok");
                            if test.fixtures[fixture_no].expected.messages.len()
                                != final_state.log.len()
                            {
                                res = format!("Expectation error (messages count doesn't match)");
                            } else {
                                &test.fixtures[fixture_no]
                                    .expected
                                    .messages
                                    .iter()
                                    .zip(final_state.log.iter().rev())
                                    .for_each(|(exp, msg)| {
                                        if exp.destination != msg.dest.0 {
                                            res = format!(
                                                "Expectation error (destination doesn't match)"
                                            );
                                        }
                                        if &exp.payload.clone().into_raw() != &msg.payload.clone().into_raw() {
                                            res = format!(
                                                "Expectation error (payload doesn't match)"
                                            );
                                        }
                                    });
                            }
                            res
                        }
                        Err(e) => {
                            total_failed += 1;
                            format!("Running error ({})", e)
                        }
                    }
                }
                Err(e) => {
                    total_failed += 1;
                    format!("Initialization error ({})", e)
                }
            };

            println!("Fixture {}: {}", test.fixtures[fixture_no].title, output);
        }
    }

    if total_failed == 0 {
        Ok(())
    } else {
        Err(anyhow!("{} tests failed", total_failed))
    }
}
