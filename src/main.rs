use std::collections::HashMap;

use libc::*;

struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>
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

    fn to_string(&self) -> String {      
        let response_line = String::from(
            self.version + 
            " " + 
            &self.status.to_string() +
            " " +
            self._status_message(self.status) +
            "\r\n"
        );

        let headers = self.headers
            .iter()
            .map(|(k, v)| {
                k + ": " + v + "\r\n"
            })
            .collect();
            
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

                // create response
                let req_contents = std::str::from_utf8_mut(&mut buffer[..read_bytes as usize]).unwrap();
                
                let Some(http_req) = parse_request(req_contents) else { return; };

                // return http response
                let content = std::fs::read_to_string(http_req.path.strip_prefix("/").unwrap()).unwrap();
                let status = String::from("HTTP/1.1 200 OK");
                let content_len = content.len().to_string();
                let res = status + "\r\n" + "Content-Length: " + &content_len + "\r\n\r\n" + &content;
                //println!("message: {}", &message);
                let write_bytes = write(client_fd, res.as_bytes().as_ptr() as *const c_void, res.len());
                if write_bytes < 0 { panic!("write to client file descriptor filed"); }

                close(client_fd);
            });
            
        }
        
    }
}
