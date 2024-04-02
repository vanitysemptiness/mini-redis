use mini_redis::{client, Result};



// this is a macro that 
#[tokio::main]
async fn main() -> Result<()> {
    /*
     * TCP connection established between running app and remote address.
     * Remote address is IP address and port number,
     * Transmission Control Protocol establishes reliable bidirectional communication
     *  between two networked devices.
     * 
     * Code looks synchronous but is not, when we see await it is not sync
     * When we make a sync operation the program execution is pauesed until completed
     * 
     * Calling an async fn returns a value representing the operation
     * cenceptually analagous to a zero-argument closure
     * 
     * Rust converts an async fn into a routine that operates async at COMPILE TIME
     * 
     * The .await operator returns the value
     */
    let mut client = client::connect("127.0.0.1:6379").await?;
    client.set("hello", "world".into()).await?;
    // get hello
    let result = client.get("hello").await?;
    println!("retrieved value from server result={:?}", result);
    println!("Hello, world!");
    Ok(())
}
