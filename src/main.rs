
use exefind::*;

fn main() -> Result<(), Error> {
    
    match RunType::new() {
        Err(errs) => {
            println!("Arg errors:");

            for err in errs {
                println!("\t{}", err);
            }

            println!("\nUse -h for help.");

            return Err(Error::ArgErr)
        },
        Ok(task) => {
            task.run()?;
        },
    }

    Ok(())
}
