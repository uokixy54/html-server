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
                let mut buffer = [0u8; 1024];
                let read_bytes = read(client_fd, buffer.as_mut_ptr() as *mut c_void, buffer.len());
                if read_bytes < 0 { panic!("read error"); }
                //println!("read_bytes: {}", std::str::from_utf8(&buffer[..read_bytes as usize]).unwrap());

                // return http response
                let message = String::from("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!");
                //println!("message: {}", &message);
                let write_bytes = write(client_fd, message.as_bytes().as_ptr() as *const c_void, message.len());
                if write_bytes < 0 { panic!("write to client file descriptor filed"); }
            });
            
        }
        
    }
}
