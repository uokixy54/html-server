use std::collections::HashMap;

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

const SELF_SOCKADDRIN: sockaddr_in = sockaddr_in {
    sin_family: AF_INET as u16,
    sin_port: htons(8080),
    sin_addr: in_addr { s_addr: INADDR_ANY },
    sin_zero: [0; 8],
};

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
            &SELF_SOCKADDRIN as *const sockaddr_in as *const sockaddr,
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
            let mut client_addr: sockaddr_in = std::mem::zeroed();
            let mut client_addr_len = std::mem::size_of::<sockaddr_in>() as u32;
            let client_fd = accept(
                socket_fd,
                &mut client_addr as *mut sockaddr_in as *mut sockaddr,
                &mut client_addr_len,
            );
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
                let raw_request = std::str::from_utf8_mut(&mut buffer[..read_bytes as usize]).unwrap();          
                let Some(http_req) = HttpRequest::parse_request(raw_request) else { return; };

                let ip = client_addr.sin_addr.s_addr;
                let ip_str = format!("{}.{}.{}.{}", ip & 0xFF, (ip >> 8) & 0xFF, (ip >> 16) & 0xFF, (ip >> 24)& 0xFF);
                println!("{}:{} {} {} {}", ip_str, ntohs(client_addr.sin_port).to_string(), http_req.method, http_req.path, http_req.version);

                // return http response
                let file_path = match http_req.path.as_str() {
                    "/" | "/ProjectStaticQuiz-" | "/ProjectStaticQuiz-/" => "ProjectStaticQuiz-/index.html",
                    _ => http_req.path.strip_prefix("/").unwrap()
                };
                let (body, status) = match std::fs::read_to_string(file_path) {
                    Ok(body) => (body, 200),
                    Err(_) => (String::from("Not Found"), 404),
                };

                let mut headers = HashMap::new();
                headers.insert(String::from("Content-Length"), body.as_bytes().len().to_string());

                let res = HttpResponse{
                    version: String::from("HTTP/1.1"),
                    status,
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
