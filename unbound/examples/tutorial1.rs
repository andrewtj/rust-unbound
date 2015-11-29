use std::process::exit;

extern crate unbound;

fn main() {
    let ctx = unbound::Context::new().unwrap();
    match ctx.resolve("www.nlnetlabs.nl", 1, 1) {
        Ok(ans) => {
            if ans.havedata() {
                for data in ans.datas() {
                    assert_eq!(data.len(), 4);
                    println!("The address is {}.{}.{}.{}",
                             data[0],
                             data[1],
                             data[2],
                             data[3]);
                }
            }
        }
        Err(err) => {
            println!("resolve error: {}", err);
            exit(1)
        }
    }
}
