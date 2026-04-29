pub struct Connection {
    socket: TcpStream,  // ← TCP connection
}

impl Connection {
    pub fn send(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> { 
        // Actually write to TCP socket
        self.socket.write_all(bytes)?;
        Ok(())
    }
    
    pub fn receive(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Actually read from TCP socket
        self.socket.read(&mut buffer)?;
        buffer
    }
    
    fn disconnect(&mut self) {
        self.socket.shutdown(Shutdown::Both)?;
    }
    
    fn get_address(&self) -> &str {
        &self.server_address
    }
}