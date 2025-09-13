use copper::dns::get_host_addr;

fn main() {
    println!("{:?}", get_host_addr("www.example.com"));
}
