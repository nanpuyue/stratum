#[cfg(test)]
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate serde_json;

#[cfg(test)]
mod stratum {
    extern crate serde;
    extern crate serde_json;

    use serde::Serialize;
    use std::io::prelude::*;
    use std::io::{self, BufReader, BufRead};
    use std::net::TcpStream;

    use self::msg::JsonToString;

    pub struct Pool {
        addr: String,
        stream: Option<TcpStream>,
    }

    mod msg {
        #[derive(Serialize, Debug)]
        pub struct Client<'a> {
            pub id: i32,
            pub method: String,
            pub params: Vec<&'a str>,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Server {
            pub id: i32,
            pub result: serde_json::Value,
            pub error: Option<Vec<String>>,
        }

        pub trait JsonToString {
            fn to_string(&self) -> serde_json::Result<String>;
        }

        impl<T: serde::Serialize> JsonToString for T {
            fn to_string(&self) -> serde_json::Result<String> {
                serde_json::to_string(&self)
            }
        }
    }

    impl Pool {
        pub fn new(addr: &str) -> Self {
            Self {
                addr: String::from(addr),
                stream: None,
            }
        }

        pub fn try_connect(&mut self) -> io::Result<&TcpStream> {
            match self.stream {
                Some(ref s) => Ok(s),
                None => {
                    self.stream = Some(TcpStream::connect(&self.addr)?);
                    Ok(self.stream.as_ref().unwrap())
                }
            }
        }

        pub fn try_send<T: Serialize>(&mut self, msg: T) -> io::Result<usize> {
            let mut data = serde_json::to_vec(&msg).unwrap();
            data.push('\n' as u8);
            Ok(self.try_connect()?.write(&data)?)
        }

        pub fn try_read(&mut self) -> io::Result<msg::Server> {
            let mut buf = String::new();
            let mut bufr = BufReader::new(self.try_connect()?);
            bufr.read_line(&mut buf).unwrap();
            println!("{}", &buf);
            let ret: msg::Server = serde_json::from_str(&buf)?;
            Ok(ret)
        }

        pub fn subscribe(&mut self) -> io::Result<usize> {
            let msg = msg::Client {
                id: 1,
                method: String::from("mining.subscribe"),
                params: vec![],
            };

            self.try_send(&msg)
        }
    }

    #[test]
    fn connect_to_tcp() {
        let mut s = Pool::new("cn.ss.btc.com:1800");
        let ret = s.try_connect();
        println!("1,{:?}", ret);
        let ret = s.subscribe();
        println!("2,{:?}", ret);
        let ret = s.try_read();
        println!("3,{:?}", ret);
    }

    #[test]
    fn serialize_json_data() {
        let msg = msg::Client {
            id: 1,
            method: String::from("mining.subscribe"),
            params: vec![],
        };
        assert_eq!(r#"{"id":1,"method":"mining.subscribe","params":[]}"#, &msg.to_string().unwrap());

        let msg = msg::Server {
            id: 2,
            result: json!(true),
            error: None,
        };
        assert_eq!(r#"{"id":2,"result":true,"error":null}"#, &msg.to_string().unwrap());

        let msg = msg::Client {
            id: 3,
            method: String::from("mining.authorize"),
            params: vec!["user1", "password"],
        };
        assert_eq!(r#"{"id":3,"method":"mining.authorize","params":["user1","password"]}"#,
                   &msg.to_string().unwrap());
    }
}
