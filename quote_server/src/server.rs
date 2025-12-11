use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};


pub fn handle_client(stream: TcpStream) {
    // клонируем stream: один экземпляр для чтения (обёрнут в BufReader), другой — для записи
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    // send initial prompt
    let _ = writer.write_all(b"Welcome to the Vault!\n");
    let _ = writer.flush();

    let mut line = String::new();
    loop {
        line.clear();
        // read_line ждёт '\n' — nc отправляет строку по нажатию Enter
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF — клиент закрыл соединение
                return;
            }
            Ok(_) => {
                let input = line.trim();
                if input.is_empty() {
                    let _ = writer.flush();
                    continue;
                }

                let mut parts = input.split_whitespace();
                let response = match parts.next() {
                    Some("STREAM") => {
                        "OK\n".to_string()
    
                    }

                    Some("RESTREAM") => {
                        "OK\n".to_string()
    
                    }

                    Some("STOP") => {
                        "OK\n".to_string()

                    }

                    _ => "BAD\n".to_string(),
                };

                // отправляем ответ и снова показываем prompt
                let _ = writer.write_all(response.as_bytes());
                let _ = writer.flush();
            }
            Err(_) => {
                // ошибка чтения — закрываем
                return;
            }
        }
    }
}
