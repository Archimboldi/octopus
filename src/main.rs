// #![windows_subsystem = "windows"]
#[macro_use]
extern crate sciter;
// use libloading::{Symbol, Library};
// use libc::*;
// use std::{ffi::CString, ptr::null_mut};
use std::thread;
use tokio::{runtime::Runtime, sync::mpsc, net::TcpStream, io::AsyncWriteExt};
use tokio_util::compat::TokioAsyncWriteCompatExt;
use once_cell::sync::Lazy;
use std::io::prelude::*;
use encoding::all::GBK;
use encoding::{Encoding, EncoderTrap};

struct Handler<'a>{
    tx: &'a mpsc::Sender<String>,
    xr: &'a mut mpsc::Receiver<String>,
    dist: String
}

impl<'a> Handler<'a>{
    fn index_sum(&mut self, lx:i32, sd:String, ed:String) -> String {
        let msg = format!("+;{};{};{}",lx,sd,ed);
        &self.tx.blocking_send(msg).unwrap();
        self.xr.blocking_recv().unwrap()
    }
    fn write(&self, d:String,e:String,f:String) -> i32 {
        let msg = format!("{},{},{}\r\n",d,e,f);
        let mut file = std::fs::OpenOptions::new().append(true).create(true).open("dat.csv").unwrap();
        // let mut reader = io::BufReader::new(&file);
        // let mut buf = String::new();
        // reader.read_to_string(&mut buf).unwrap();
        
        if let Ok(_) = file.write(msg.as_bytes()){
            return 0;
        }
        1
    }
    fn exporto(&self, o:i32) {
        if o == 0 {
            std::fs::copy("./temp.csv", format!("{}/挂接查询.csv",&self.dist)).unwrap();
        }else if o == 1 {
            std::fs::copy("./dat.csv", format!("{}/人工记录.csv",&self.dist)).unwrap();
        }
    }
}
impl<'a> sciter::EventHandler for Handler<'a> {
    dispatch_script_call! {
        fn index_sum(i32, String, String);
        fn exporto(i32);
        fn write(String,String,String);
    }
}

fn main() {
    use tiberius::{AuthMethod, Client, Config};
    let (tx, mut rx) = mpsc::channel::<String>(1);
    let (xt, mut xr) = mpsc::channel::<String>(1);
    let sync_code = thread::spawn(move || {
        // let html = include_bytes!("../ui/clock.htm");

        let resouces = include_bytes!("../ui.rc");

        let mut frame = sciter::WindowBuilder::main_window().with_size((378, 340)).with_pos((540,256)).create();
        
        // let load = format!("file://{}/ui/clock.htm", std::env::current_dir().unwrap().to_str().unwrap());
        // frame.load_html(html, Some(&load));
        
        //packfolder.exe ui ui.rc -binary
        frame.archive_handler(resouces).expect("Invalid archive");
        frame.load_file("this://app/clock.htm");

        let dist: String = std::env::args().nth(1).unwrap();
        let handler = Handler{tx:&tx, xr: &mut xr,dist:dist};
        frame.event_handler(handler);
        
        frame.run_app();
        tx.blocking_send("over".to_string()).unwrap();
    });

    Runtime::new().unwrap().block_on(async move {
        static CONN_STR: Lazy<String> = Lazy::new(|| {
            std::env::var("TIBERIUS_CONNECTION_STRING").unwrap_or_else(|_| {
                "server=tcp:localhost,1433;IntegratedSecurity=true;TrustServerCertificate=true".to_owned()
            })
        });
        let mut config = Config::from_ado_string(&CONN_STR).unwrap();
        config.database("cb");
        config.authentication(AuthMethod::sql_server("sa", "12345"));
        let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
        tcp.set_nodelay(true).unwrap();
        let mut client = Client::connect(config, tcp.compat_write()).await.unwrap();
        loop {
            if let Some(c) = rx.recv().await{
                if c.starts_with("+") {
                    let args:Vec<&str> = c.splitn(4,";").collect();
                    let que = format!("SELECT PID,DID,TITLE from dbo.E_FILE{} WHERE STATUS = 0 AND CREATETIME BETWEEN '{}' AND '{}'", args[1],args[2],args[3]);
                    let stream = client
                        .query(que,
                            &[&1, &2, &3],
                        )
                        .await
                        .unwrap();
                    let rowsets = stream.into_results().await.unwrap();
                    let mut ts = 0;
                    let mut zys = 0;
                    let mut nys = 0;
                    let mut ys0 = 0;

                    if let Some(rows) = rowsets.get(0) {
                        let mut file = tokio::fs::File::create("temp.csv").await.unwrap();
                        for row in rows {
                            ts = ts+1;
                            let pid =row.get::<i32, _>(0).unwrap();
                            let dh = row.get::<&str, _>(2).unwrap();
                            let u = format!("SELECT YS,TITLE from dbo.D_FILE{} WHERE STATUS = 0 AND DID = {}", args[1], pid);
                            let resu = client
                                .query(u, 
                                    &[&1, &2]
                                )
                                .await
                                .unwrap();
                                let res = resu.into_results().await.unwrap();
                                if let Some(re) = res.get(0){
                                    for r in re {
                                        let ys = r.get::<i32, _>(0);
                                        let title = r.get::<&str, _>(1).unwrap();
                                        let mut buf = String::new();
                                        if ys != None {
                                            let s = ys.unwrap();
                                            buf = format!("{},{},{}\r\n",dh,s,title);
                                            if s == 0 {
                                                ys0 = ys0+1;
                                            }else {
                                                zys = zys + s;
                                            }
                                        }else{
                                            nys = nys+1;
                                            buf = format!("{},,{},\r\n",dh,title)
                                        }
                                        file.write_all(&GBK.encode(&buf, EncoderTrap::Strict).unwrap()).await.unwrap();
                                        buf.clear();
                                    }
                                }
                        }
                    }
                    let msg = format!("-{} - {}，挂接数：{}件，总页数：{}页，页数为空：{}条，页数为0：{}条", args[2],args[3],ts, zys, nys, ys0);
                    xt.send(msg).await.unwrap();
                }else if c == "over".to_string() {
                    break;
                }
            };
        }

  
    });
    
    sync_code.join().unwrap()
}

// fn _token() -> bool {
//     const LIB: &'static str = "token.dll";
//     type Find=unsafe fn(*const c_uchar, *mut c_int) -> i32;
//     type Open=unsafe fn(*const *mut c_void, *const c_uchar, c_int) -> i32;
//     type Verify=unsafe fn(*const c_void, c_int, *const c_uchar) -> i32;
//     type Close=unsafe fn(*const c_void) -> i32;
//     unsafe {
//         let lib = Library::new(LIB).expect("请保证系统文件完整！");
//         let l = std::fs::read(LIB).unwrap();
//         if l.len() != 87552 {
//             return false;
//         }
//         let et_find: Symbol<Find> = lib.get(b"et_FindToken").unwrap();
//         let mut count = 0;
//         let id = CString::new("64761549").unwrap();
//         et_find(id.as_bytes().as_ptr(), &mut count);
//         if count>0 {
//             let handle: *mut c_void = null_mut();
//             let et_open: Symbol<Open> = lib.get(b"et_OpenToken").unwrap();
//             if et_open(&handle, id.as_bytes().as_ptr(), 1) == 0 {
//                 let et_verify: Symbol<Verify> = lib.get(b"et_Verify").unwrap();
//                 let word = CString::new("19940501EDFABCAB").unwrap();
//                 let v = et_verify(handle as *const c_void, 0, word.as_bytes().as_ptr());
//                 let et_close: Symbol<Close> = lib.get(b"et_CloseToken").unwrap();
//                 et_close(handle);
//                 if v == 0{
//                     true
//                 }else{
//                     false
//                 }
//             }else{
//                 false
//             }
//         }else{
//             false
//         }
//     }
// }