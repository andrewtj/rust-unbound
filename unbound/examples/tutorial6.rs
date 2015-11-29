use std::process::exit;
extern crate unbound;

fn main() {
    let ctx = unbound::Context::new().unwrap();
    if let Err(err) = ctx.resolvconf_path("/etc/resolv.conf") {
        println!("error reading resolv.conf: {}", err);
        exit(1)
    }
    if let Err(err) = ctx.hosts_path("/etc/hosts") {
        println!("error reading hosts: {}", err);
        exit(1)
    }
    if let Err(err) = ctx.add_ta_file("keys") {
        println!("error adding keys: {}", err);
        exit(1)
    }
    match ctx.resolve("www.nlnetlabs.nl", 1, 1) {
        Err(err) => {
            println!("resolve error: {}", err);
            exit(1)
        }
        Ok(ans) => {
            for data in ans.datas() {
                assert_eq!(data.len(), 4);
                println!("The address is {}.{}.{}.{}",
                         data[0],
                         data[1],
                         data[2],
                         data[3]);
            }
            if ans.secure() {
                println!("Result is secure")
            } else if let Some(reason) = ans.why_bogus() {
                println!("Result is bogus: {}", reason);
            } else {
                println!("Result is insecure");
            }
        }
    }
}
