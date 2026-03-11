use libc::*;
fn main() {
    unsafe {
        // create ipv4 socket
        let socket_fd = socket(AF_INET, SOCK_STREAM, 0);
        if socket_fd < 0 { panic!("socket error"); }

        // bind socket to port 8080
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(8080),
            sin_addr: in_addr { s_addr: INADDR_ANY },
            sin_zero: [0; 8],
        };
        let bind_result = bind(
            socket_fd,
            &addr as *const sockaddr_in as *const sockaddr,
            std::mem::size_of::<sockaddr_in>() as u32
        );
        if bind_result < 0 { panic!("bind error"); }

        // listen
        let listen_result = listen(socket_fd, 128);
        if listen_result < 0 { panic!("listen error"); }

        loop {
            // accept
            let client_fd = accept(socket_fd, std::ptr::null_mut(), std::ptr::null_mut());
            println!("client file descriptor is {}", client_fd);
            if client_fd < 0 { panic!("accept failed"); }

            std::thread::spawn(move || {
                // get http request
                let mut buffer = [0u8; 8192];
                let read_bytes = read(client_fd, buffer.as_mut_ptr() as *mut c_void, buffer.len());
                if read_bytes < 0 { panic!("read error"); }
                let req_content = std::str::from_utf8_mut(&mut buffer[..read_bytes as usize]).unwrap();
                let req_line: Vec<&str> = req_content.split(" ").collect();
                if req_line.len() < 2 { return; }
                let path = req_line[1];
                println!("path is {}", path.strip_prefix("/").unwrap());

                // return http response
                let content = std::fs::read_to_string(path.strip_prefix("/").unwrap()).unwrap();
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
