use std::{collections::HashMap, ops::Add};

use libc::*;

struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>
}
impl HttpRequest {
    fn parse_request(raw_request: &str) -> Option<Self> {
        // parse request line
        let request_line: Vec<&str> = raw_request.lines().next()?.split(" ").collect();
    
        if request_line.get(0) != Some(&"GET") {
            return None;
        }
    
        if let None = request_line.get(1) {
            return None;
        }
    
        if let None = request_line.get(2) {
            return None;
        }
    
        // parse request headers
        let mut headers = HashMap::new();
        raw_request
            .lines()
            .skip(1)
            .for_each(|line| {
                if let Some((k, v)) = line.split_once(": ") {
                    headers.insert(k.to_string(), v.to_string());
                }
            });
    
        Some(Self {
            method: request_line[0].to_string(),
            path: request_line[1].to_string(),
            version: request_line[2].to_string(),
            headers,
        })
    
    }
}

struct HttpResponse {
    version: String,
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}
impl HttpResponse {
    fn _status_message(code: u16) -> &'static str {
        match code {
            200 => "OK",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown Error",
        }
    }

    fn create_response_contents(&self) -> String {      
        let response_line = format!("{} {} {}\r\n", self.version, self.status.to_string(), Self::_status_message(self.status));

        let mut headers = String::new();
        for (k, v) in self.headers.iter() {
            headers.push_str(&format!("{}: {}\r\n", k, v));
        }

        format!("{}{}\r\n{}", response_line, headers, self.body)
    }
}

const SOCKADDR_CONF: sockaddr_in = sockaddr_in {
    sin_family: AF_INET as u16,
    sin_port: htons(8080),
    sin_addr: in_addr { s_addr: INADDR_ANY },
    sin_zero: [0; 8],
};

fn parse_request(raw_request: &str) -> Option<HttpRequest> {
    // parse request line
    let request_line: Vec<&str> = raw_request.lines().next()?.split(" ").collect();

    if request_line.get(0) != Some(&"GET") {
        return None;
    }

    if let None = request_line.get(1) {
        return None;
    }

    if let None = request_line.get(2) {
        return None;
    }

    // parse request headers
    let mut headers = HashMap::new();
    raw_request
        .lines()
        .skip(1)
        .for_each(|line| {
            if let Some((k, v)) = line.split_once(": ") {
                headers.insert(k.to_string(), v.to_string());
            }
        });

    Some(HttpRequest {
        method: request_line[0].to_string(),
        path: request_line[1].to_string(),
        version: request_line[2].to_string(),
        headers,
    })

}

fn main() {
    unsafe {
        // create ipv4 socket
        let socket_fd = socket(AF_INET, SOCK_STREAM, 0);
        if socket_fd == -1 { 
            std::process::exit(1);
        }

        // bind socket to port 8080
        let bind_result = bind(
            socket_fd,
            &SOCKADDR_CONF as *const sockaddr_in as *const sockaddr,
            std::mem::size_of::<sockaddr_in>() as u32
        );
        if bind_result == -1 {
            std::process::exit(1);
        }

        // listen
        let listen_result = listen(socket_fd, 128);
        if listen_result == -1 {
            std::process::exit(1);
        }

        loop {
            // accept
            let client_fd = accept(socket_fd, std::ptr::null_mut(), std::ptr::null_mut());
            if client_fd == -1 {
                eprintln!("accept failed: {}", std::io::Error::last_os_error());
                continue;
            }

            std::thread::spawn(move || {
                // get http request
                let mut buffer = [0u8; 1024];
                let read_bytes = read(client_fd, buffer.as_mut_ptr() as *mut c_void, buffer.len());
                if read_bytes < 0 {
                    eprintln!("read failed: {}", std::io::Error::last_os_error());
                    return;
                }

                // parse http request
                let raq_request = std::str::from_utf8_mut(&mut buffer[..read_bytes as usize]).unwrap();          
                let Some(http_req) = HttpRequest::parse_request(raq_request) else { return; };

                // return http response
                let mut body = String::new();
                match http_req.path.as_str() {
                    "/" => { body = std::fs::read_to_string("ProjectStaticQuiz-/index.html").unwrap(); }
                    "/ProjectStaticQuiz-" => { body = std::fs::read_to_string("ProjectStaticQuiz-/index.html").unwrap(); },
                    "/ProjectStaticQuiz-/" => { body = std::fs::read_to_string("ProjectStaticQuiz-/index.html").unwrap(); },
                    _ => { body = std::fs::read_to_string(http_req.path.strip_prefix("/").unwrap()).unwrap(); }
                }

                let mut headers = HashMap::new();
                headers.insert(String::from("Content-Length"), body.as_bytes().len().to_string());

                let res = HttpResponse{
                    version: String::from("HTTP/1.1"),
                    status: 200,
                    headers,
                    body,
                };

                let write_bytes = write(
                    client_fd,
                    res.create_response_contents().as_bytes().as_ptr() as *const c_void,
                    res.create_response_contents().as_bytes().len()
                );
                if write_bytes < 0 { panic!("write to client file descriptor filed"); }

                close(client_fd);
            });
            
        }
        
    }
}
