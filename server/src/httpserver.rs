use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, error, info};
use rustls::pki_types::PrivateKeyDer;
use rustls::ServerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

enum HTTPStatus {
    Ok = 200, NotFound = 404, MethodNotAllowed = 405,
}

impl HTTPStatus {
    fn get_message(&self) -> String {
        match self {
            HTTPStatus::Ok => {String::from("OK")}
            HTTPStatus::NotFound => {String::from("Not Found")}
            HTTPStatus::MethodNotAllowed => {String::from("Method Not Allowed")}
        }
    }
}

pub struct HTTPServer {}

#[derive(Debug)]
pub struct HTTPGetReq {
    pub path: Option<String>,
    pub headers: HashMap<String, String>,
}

impl HTTPServer {
    pub async fn start(addr: &str, content_path: &str, is_secure: bool) -> Result<(), Box<dyn Error>> {
        let files = HTTPServer::read_files(content_path, &String::from(""))?;
        debug!("Files: {:?}", &files.keys());
        let acceptor = if is_secure {
            let tls_config = Arc::new(HTTPServer::create_tls_config()?);
            Some(TlsAcceptor::from(tls_config))
        } else {None};
        let listener = TcpListener::bind(addr).await?;
        info!("HTTPServer started on {addr}");

        let files_arc = Arc::new(files);
        loop {
            let (stream, addr) = listener.accept().await?;
            let files_clone = files_arc.clone();
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let res = if is_secure {
                    HTTPServer::handle_secure_connection(stream, &addr, files_clone, acceptor.unwrap()).await
                } else {
                    let (mut reader, mut writer) = tokio::io::split(stream);
                    HTTPServer::handle_connection(&mut reader, &mut writer, &addr, files_clone).await
                };
                if let Err(e) = res {
                    if e.to_string().contains("peer closed connection") {
                        debug!("Connection closed")
                    } else {
                        error!("Failed to handle request: {:?}", e.to_string());
                    }
                }
            });
        }
    }

    fn read_files(content_path: &str, prefix: &String) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        let mut result = HashMap::new();
        info!("Scan dir {content_path}");
        for entry in fs::read_dir(content_path)?.flatten() {
            match HTTPServer::read_dir_entry(prefix, &entry) {
                Ok(res) => {
                    result.extend(res.into_iter());
                }
                Err(e) => {
                    error!("Failed to read entry {:?}: {:?}", &entry, e);
                }
            }
        }
        Ok(result)
    }

    fn read_dir_entry(prefix: &String, entry: &DirEntry) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        let filename = entry.file_name().into_string().map_err(|s| format!("Invalid OsString {:?}", s))?;
        if filename.contains("code-workspace") || filename.starts_with('.') {
            Ok(HashMap::new())
        } else {
            let filename_with_prefix = format!("{}/{}", prefix, filename);
            if entry.path().is_dir() {
                let path = entry.path();
                HTTPServer::read_files(path.to_str().unwrap_or_default(), &filename_with_prefix)
            } else {
                let mut result = HashMap::with_capacity(1);
                let data = fs::read(entry.path())?;
                info!("Read {}: {} bytes", &filename_with_prefix, data.len());
                result.insert(filename_with_prefix.strip_prefix('/').map(String::from).unwrap_or(filename_with_prefix), data);
                Ok(result)
            }
        }
    }

    async fn handle_secure_connection(stream: TcpStream, addr: &SocketAddr,
                                      files: Arc<HashMap<String, Vec<u8>>>, acceptor: TlsAcceptor) -> Result<(), Box<dyn Error>> {
        let stream = acceptor.accept(stream).await?;
        let (mut reader, mut writer) = tokio::io::split(stream);
        HTTPServer::handle_connection(&mut reader, &mut writer, addr, files).await
    }

    async fn handle_connection<R, W>(reader: &mut R, writer: &mut W, addr: &SocketAddr,
                               files: Arc<HashMap<String, Vec<u8>>>) -> Result<(), Box<dyn Error>>
        where R: AsyncReadExt + Unpin, W: AsyncWriteExt + Unpin {
        let mut keep_alive = true;
        while keep_alive {
            let req = HTTPServer::read_http_req(reader).await?;
            debug!("{:?}", &req);
            keep_alive = if let Some(connection) = req.headers.get("Connection") {
                connection.as_str() == "keep-alive"
            } else {
                false
            };
            let response = if let Some(path) = req.path {
                info!("{:?}: GET '{}'", addr.ip(), &path);
                let filename = path.strip_prefix('/').map(String::from).unwrap_or(path);
                let filename = if filename.is_empty() {String::from("index.html")} else {filename};
                if let Some(data) = files.get(filename.as_str()) {
                    HTTPServer::generate_response(HTTPStatus::Ok, filename.as_str(), data.as_slice(), keep_alive)
                } else {
                    HTTPServer::generate_response(HTTPStatus::NotFound, "", &[0], keep_alive)
                }
            } else {
                HTTPServer::generate_response(HTTPStatus::MethodNotAllowed, "", &[0], keep_alive)
            };
            writer.write_all(&response).await?;
            writer.flush().await?;
        }
        Ok(())
    }

    fn generate_response(status: HTTPStatus, filename: &str, data: &[u8], keep_alive: bool) -> Vec<u8> {
        let mut headers = HashMap::new();
        headers.insert("Content-Length", format!("{}", data.len()));
        headers.insert("Content-Type", String::from(HTTPServer::decode_content_type(filename)));
        headers.insert("Server", String::from("EasymundHTTP"));
        headers.insert("Connection", String::from(if keep_alive {"keep-alive"} else {"close"}));
        debug!("Response headers {:?}", &headers);

        let mut result = Vec::new();
        let status_message = status.get_message();
        let status_line = format!("HTTP/1.1 {} {}\r\n", status as u16, status_message);
        result.extend_from_slice(status_line.as_bytes());
        for (key, value) in headers {
            let header_line = format!("{}: {}\r\n", key, value);
            result.extend_from_slice(header_line.as_bytes());
        }
        result.extend_from_slice("\r\n".as_bytes());
        result.extend_from_slice(data);
        result
    }

    fn decode_content_type(filename: &str) -> &str {
        if filename.ends_with("html") {
            "text/html"
        } else if filename.ends_with("js") {
            "application/javascript"
        } else if filename.ends_with("wasm") {
            "application/wasm"
        } else {
            "text/plain"
        }
    }

    pub async fn read_http_req<T>(stream: &mut T) -> Result<HTTPGetReq, Box<dyn Error>>
        where T: AsyncReadExt + Unpin {
        let mut request = Vec::new();
        let mut new_lines_counter = 0;
        while new_lines_counter < 4 {
            let ch = stream.read_u8().await? as char;
            if ch == '\r' || ch == '\n' {
                new_lines_counter += 1;
            } else {
                new_lines_counter = 0;
            }
            request.push(ch as u8);
        }
        let request = String::from_utf8(request)?;
        Ok(HTTPServer::parse_req(request))
    }

    fn parse_req(req: String) -> HTTPGetReq {
        let mut path = None;
        let mut headers = HashMap::new();
        for line in req.split("\r\n") {
            if let Some(path_str) = line.strip_prefix("GET") {
                let path_str = path_str.trim();
                let path_str = match path_str.find(' ') {
                    Some(i) => path_str.split_at(i).0,
                    None => path_str
                };
                path = Some(String::from(path_str));
            } else if let Some(i) = line.find(':') {
                let (key, value) = line.split_at(i);
                let value = value.chars().skip(1).collect::<String>();
                headers.insert(String::from(key.trim()), String::from(value.trim()));
            }
        }
        HTTPGetReq {path, headers}
    }

    pub fn create_tls_config() -> Result<ServerConfig, Box<dyn Error>> {
        let key_file = &mut BufReader::new(File::open("cert/privkey.pem")?);
        let mut key_option = None;
        for item in std::iter::from_fn(|| rustls_pemfile::read_one(key_file).transpose()) {
            match item.unwrap() {
                rustls_pemfile::Item::Pkcs8Key(key) => key_option = Some(PrivateKeyDer::from(key)),
                _ => error!("unhandled item"),
            }
        }
        if key_option.is_none() {
            return Err("No private key")?;
        }

        let cert_file = &mut BufReader::new(File::open("cert/fullchain.pem")?);
        let cert = rustls_pemfile::certs(cert_file).collect::<Result<Vec<_>, _>>()?;
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key_option.unwrap())?;
        Ok(config)
    }
}