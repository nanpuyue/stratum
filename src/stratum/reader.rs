use crate::util::{hex_to, ToHex};

use super::*;

impl Pool {
    pub(super) fn reader(&mut self) -> impl Future<Item = (), Error = ()> + Send {
        let authorized = self.authorized.clone();
        let xnonce = self.xnonce.clone();
        let submitted_nonce = self.submitted_nonce.clone();
        let work_sender = self.work_channel.0.clone();
        let work_notify = self.work_notify.clone();
        let vermask = self.vermask.clone();
        let diff = self.diff.clone();
        self.receiver().for_each(move |line| {
            if let Ok(s) = serde_json::from_str::<Action>(&line) {
                match s.params {
                    Params::Work(w) => {
                        info!("=> received new work!");
                        let work_notify = work_notify.clone();
                        tokio::spawn(work_sender.clone().send(w).then(move |_| {
                            work_notify.notify();
                            Ok(())
                        }));
                    }
                    Params::Num([n]) => {
                        if s.method == "mining.set_difficulty" {
                            info!("=> set difficulty: {}!", &n);
                            *diff.lock().unwrap() = n;
                        }
                    }
                    Params::TMask(mask) => {
                        if s.method == "mining.set_version_mask" {
                            if mask.len() == 1 {
                                let mask = mask[0];
                                info!("=> set vermask: 0x{}!", mask.to_be_bytes().to_hex());
                                *vermask.lock().unwrap() = Some(mask);
                            } else {
                                warn!("=> unknown vermask: {:?}!", mask);
                            }
                        }
                    }
                    _ => info!("=> {}: {:?}", s.method, s.params),
                }
            } else if let Ok(s) = serde_json::from_str::<Respond>(&line) {
                match s.result {
                    ResultOf::Authorize(r) => {
                        let result = r.unwrap_or(false);

                        match s.id {
                            Some(2)
                                if {
                                    let mut authorized = authorized.lock().unwrap();
                                    if !authorized.1 {
                                        if result {
                                            authorized.1 = true;
                                            info!("=> authorized successfully!");
                                        } else {
                                            info!("=> authorized failed!");
                                        }
                                        true
                                    } else {
                                        false
                                    }
                                } => {}
                            Some(id)
                                if {
                                    if let Some(nonce) = submitted_nonce.lock().unwrap()
                                        [(id & 0b111) as usize]
                                        .take()
                                    {
                                        if result {
                                            info!("=> submitted nonce 0x{:08x} accepted!", nonce);
                                        } else {
                                            info!("=> submitted nonce 0x{:08x} rejected!", nonce);
                                        }
                                        true
                                    } else {
                                        false
                                    }
                                } => {}
                            _ => warn!("unknown respond: {}!", line),
                        }
                    }
                    ResultOf::Subscribe(_, xnonce1, xnonce2_size) => {
                        info!(
                            "=> set xnonce1: 0x{}, xnonce2_size: {}!",
                            xnonce1.to_hex(),
                            xnonce2_size
                        );
                        let mut xnonce = xnonce.lock().unwrap();
                        xnonce.0 = xnonce1;
                        xnonce.1 = xnonce2_size;
                    }
                    ResultOf::Configure(r) => {
                        if let Some(result) = r.get("version-rolling") {
                            if let serde_json::Value::Bool(result) = result {
                                if *result {
                                    let mask = hex_to::u32(r.get("version-rolling.mask").unwrap())
                                        .unwrap();
                                    info!("=> set vermask: 0x{}!", mask.to_be_bytes().to_hex());
                                    *vermask.lock().unwrap() = Some(mask);
                                } else {
                                    info!("=> the pool does not support version-rolling!");
                                }
                            } else if let serde_json::Value::String(e) = result {
                                info!("=> the pool does not support version-rolling: {:?}!", e);
                            }
                        }
                    }
                }
            }
            Ok(())
        })
    }
}
