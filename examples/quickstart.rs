use aimx::{availability, respond, Error};

fn main() -> Result<(), Error> {
    futures_executor::block_on(async {
        if let Err(reason) = availability() {
            eprintln!("Apple Intelligence is not available: {reason}");
            return Ok(());
        }

        let response = respond("Explain Rust ownership in one sentence.").await?;
        println!("{response}");

        Ok(())
    })
}
