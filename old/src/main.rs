// extern crate sysbar;

use pingr::{Ping, PingResult};

fn do_stuff() -> PingResult<()> {
    let mut ping = Ping::new();
    ping.set_timeout(5.0)?; // timeout of 5.0 seconds
    ping.add_host("192.168.1.1")?;
    ping.add_host("google.com")?; // fails here if socket can't be created
    ping.add_host("foo.com")?; // fails here if socket can't be created
    // ping.add_host("::1")?; // IPv4 / IPv6 addresses OK
    for resp in ping.send()? { // waits for responses from all, or timeout
        if resp.dropped > 0 {
            println!("Host:\t{}\t\tNo Response", resp.hostname);
        } else {
            println!("Host:\t{}", resp.hostname);
            println!("{}", resp);
        }
    }
    Ok(())
}

fn main() {
    match do_stuff() {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e);
            ()
        }
    }
}


//use wifi::scanner;
//use wifilocation;

// fn main() {
//    println!("{:?}", wifilocation::get_location(wifilocation::get_towers()));

// let network_info = scanner::get_info();
// println!("{:?}", network_info);

// let mut bar = sysbar::Sysbar::new("pingr");
//
// bar.add_item(
//     "run test",
//     Box::new(move || {
//         run_test();
//     }),
// );
//
// bar.add_quit_item("Quit");
//
// bar.display();
// }