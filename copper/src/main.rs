use copper::dns::CLOUDFLARE_DNS;
use copper::dns::get_host_addr;
use copper::http::HttpRequest;
use copper::http::HttpResponse;
use copper::url::Url;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

fn main() {
    const addr: &str = "http://www.example.com/";

    let url: Url<'_> = addr.try_into().unwrap();
    println!("{:?}", url);

    let ip = get_host_addr(url.host(), CLOUDFLARE_DNS).unwrap();
    println!("{:?}", ip);

    let mut http_request = HttpRequest::new("GET", url);
    let http_message = http_request.as_bytes();
    println!("{}", str::from_utf8(&http_message).unwrap());

    let mut tcp = TcpStream::connect((ip, 80)).unwrap();
    tcp.write(&http_message).unwrap();
    let mut response_message = Vec::new();
    tcp.read_to_end(&mut response_message).unwrap();
    let response = HttpResponse::from_bytes(&response_message).unwrap();
    println!("{:?}", response);
}
